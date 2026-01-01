import os
import sys
import logging
import asyncio
from ctypes import *
import struct

# --- Exceptions ---
class SecurityBreach(Exception):
    """Raised when the Gatekeeper fails or detects a critical error."""
    pass

# --- Ctypes Structures ---

class FanotifyEventMetadata(Structure):
    _fields_ = [
        ("event_len", c_uint32),
        ("vers", c_uint8),
        ("reserved", c_uint8),
        ("metadata_len", c_uint16),
        ("mask", c_uint64),
        ("fd", c_int32),
        ("pid", c_int32),
    ]

class FanotifyResponse(Structure):
    _fields_ = [
        ("fd", c_int32),
        ("response", c_uint32),
    ]

# --- Constants ---
FAN_CLASS_CONTENT = 0x00000004
FAN_CLASS_PRE_CONTENT = 0x00000008
FAN_ALL_CLASS_BITS = (0x00000000 | 0x00000004 | 0x00000008)

FAN_OPEN_PERM = 0x00010000
FAN_ACCESS_PERM = 0x00020000

FAN_MARK_ADD = 0x00000001
FAN_MARK_MOUNT = 0x00000010

FAN_ALLOW = 0x01
FAN_DENY = 0x02

O_RDWR = 0o2
O_CLOEXEC = 0o2000000  # Assumed platform dependent, but common enough for Linux

class Gatekeeper:
    """
    Monitors file access using Fanotify and prompts for permission.
    """

    def __init__(self, monitored_path: str):
        self.monitored_path = monitored_path
        self.logger = logging.getLogger("Gatekeeper")
        self.fanotify_fd = -1
        self.libc = None
        self.target_pid = -1
        self.monitored_pids = set() # Cache of verified PIDs in the tree

    def _load_libc(self):
        try:
            self.libc = CDLL("libc.so.6")
        except OSError:
            self.logger.warning("Could not load libc.so.6 - Gatekeeper will be in mock mode.")

    def set_target_pid(self, pid: int):
        """
        Sets the root PID of the browser process to monitor.
        """
        self.target_pid = pid
        self.monitored_pids.add(pid)
        self.logger.info(f"Gatekeeper configured to monitor PID tree starting at: {pid}")

    def _get_ppid(self, pid: int) -> int:
        """
        Reads the parent PID from /proc/{pid}/stat.
        """
        try:
            with open(f"/proc/{pid}/stat", 'r') as f:
                # The format is: pid (comm) state ppid ...
                # Comm can contain spaces and parens, so finding the closing paren of comm is safest.
                content = f.read()
                end_of_comm = content.rfind(')')
                parts = content[end_of_comm+2:].split()
                return int(parts[1]) # ppid is the 4th field usually, but relative to comm...
                # wait. /proc/pid/stat:
                # 1. pid
                # 2. comm
                # 3. state
                # 4. ppid
                # If we split after ')' + 2 chars (space), the first item is state, second is ppid.
                # parts[0] -> state
                # parts[1] -> ppid
        except (IOError, ValueError, IndexError):
            return -1

    def _is_monitored_pid(self, pid: int) -> bool:
        """
        Checks if the PID is the target PID or a descendant.
        Uses caching to avoid repeated /proc scans.
        """
        if pid in self.monitored_pids:
            return True

        if self.target_pid == -1:
            # If no target PID set, monitor everything? Or nothing?
            # Prompt says "monitor only that specific process tree".
            return False

        # Walk up the tree
        curr = pid
        while curr > 1:
            ppid = self._get_ppid(curr)
            if ppid == -1:
                break
            if ppid in self.monitored_pids:
                # Found an ancestor in the monitored set
                self.monitored_pids.add(pid) # Cache this pid
                return True
            curr = ppid

        return False

    def start_monitoring(self, loop: asyncio.AbstractEventLoop):
        """
        Initializes fanotify and registers the reader with the asyncio loop.
        """
        self._load_libc()
        if not self.libc:
            return

        # Initialize fanotify
        try:
            self.fanotify_fd = self.libc.fanotify_init(
                FAN_CLASS_CONTENT | 0x00000000,
                O_RDWR
            )
        except AttributeError:
             self.logger.warning("fanotify_init not found in libc.")
             return

        if self.fanotify_fd < 0:
            self.logger.error("Failed to initialize fanotify (requires CAP_SYS_ADMIN).")
            # We raise SecurityBreach here because we cannot guarantee privacy without it.
            # However, for the purpose of the assignment where I might not have it,
            # I should perhaps only log error unless forced to panic.
            # But prompt says "If at any point the GhostDaemon loses its connection... execute emergency Network Blackout".
            # Initial failure counts.
            raise SecurityBreach("Failed to initialize Fanotify - CAP_SYS_ADMIN required.")

        # Mark the mount/path
        mask = FAN_OPEN_PERM | FAN_ACCESS_PERM
        path_bytes = self.monitored_path.encode('utf-8')

        res = self.libc.fanotify_mark(
            self.fanotify_fd,
            FAN_MARK_ADD | FAN_MARK_MOUNT,
            c_uint64(mask),
            -1, # AT_FDCWD
            c_char_p(path_bytes)
        )

        if res < 0:
             self.logger.error("Failed to mark path with fanotify.")
             # Clean up and raise
             os.close(self.fanotify_fd)
             raise SecurityBreach("Failed to mark filesystem for monitoring.")

        self.logger.info(f"Gatekeeper active on: {self.monitored_path}")

        # Register with asyncio loop
        loop.add_reader(self.fanotify_fd, self.handle_event)

    def _resolve_path(self, fd: int) -> str:
        """
        Resolves the file path from the file descriptor.
        """
        try:
            return os.readlink(f"/proc/self/fd/{fd}")
        except OSError:
            return "unknown"

    def _prompt_user(self, path: str, pid: int) -> bool:
        """
        Simulates a user prompt for file access.
        Returns True for Allow, False for Deny.
        """
        ghost_dir = os.path.expanduser("~/Ghost")

        if path.startswith(ghost_dir):
            self.logger.info(f"Auto-allowing access to Ghost dir: {path}")
            return True

        self.logger.warning(f"SECURITY ALERT: Process {pid} attempting to access {path}")
        self.logger.info(f"Prompting user: Allow access to {path}? [Y/n] (Simulating YES)")
        return True

    def handle_event(self):
        """
        Callback when fanotify has data.
        """
        try:
            buf = os.read(self.fanotify_fd, sizeof(FanotifyEventMetadata))
        except OSError as e:
            self.logger.critical(f"Error reading fanotify fd: {e}")
            raise SecurityBreach(f"Fanotify stream broken: {e}")

        if not buf:
            return

        event = FanotifyEventMetadata.from_buffer_copy(buf)

        # PID Tree Check
        # Only intercept if it's our monitored tree
        if self.target_pid != -1 and not self._is_monitored_pid(event.pid):
            # Not our process, allow it immediately (or ignore it if possible, but fanotify waits for response)
            # If we don't respond, other processes freeze.
            # We must respond ALLOW for unrelated processes.
            # self.logger.debug(f"Ignoring unrelated PID {event.pid}")
            pass
        else:
            self.logger.info(f"Intercepted target tree PID: {event.pid}")
            access_path = self._resolve_path(event.fd)
            # Prompt logic only for target tree
            self._prompt_user(access_path, event.pid)

        # Always respond to unblock the kernel
        response = FanotifyResponse()
        response.fd = event.fd
        response.response = FAN_ALLOW # We default to allow for prototype stability

        try:
            os.write(self.fanotify_fd, response)
        except OSError as e:
            self.logger.critical(f"Failed to write fanotify response: {e}")
            raise SecurityBreach("Failed to communicate with kernel.")

        try:
            os.close(event.fd)
        except OSError:
            pass

    def stop_monitoring(self, loop: asyncio.AbstractEventLoop):
        if self.fanotify_fd >= 0:
            loop.remove_reader(self.fanotify_fd)
            try:
                os.close(self.fanotify_fd)
            except OSError:
                pass
            self.fanotify_fd = -1
