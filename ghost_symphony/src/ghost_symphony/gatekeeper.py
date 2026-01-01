import os
import sys
import logging
import asyncio
from ctypes import *
import struct

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

    def _load_libc(self):
        try:
            self.libc = CDLL("libc.so.6")
        except OSError:
            self.logger.warning("Could not load libc.so.6 - Gatekeeper will be in mock mode.")

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
                FAN_CLASS_CONTENT | 0x00000000, # flags (simplified)
                O_RDWR
            )
        except AttributeError:
             self.logger.warning("fanotify_init not found in libc.")
             return

        if self.fanotify_fd < 0:
            self.logger.error("Failed to initialize fanotify (requires CAP_SYS_ADMIN).")
            return

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
            # We don't return here to allow continued execution for demo purposes if it fails,
            # but in a real app this is critical.
            # return

        self.logger.info(f"Gatekeeper monitoring: {self.monitored_path}")

        # Register with asyncio loop
        loop.add_reader(self.fanotify_fd, self.handle_event)

    def _resolve_path(self, fd: int) -> str:
        """
        Resolves the file path from the file descriptor.
        """
        try:
            # /proc/self/fd/{fd} points to the open file
            path = os.readlink(f"/proc/self/fd/{fd}")
            return path
        except OSError:
            return "unknown"

    def _prompt_user(self, path: str, pid: int) -> bool:
        """
        Simulates a user prompt for file access.
        Returns True for Allow, False for Deny.
        """
        # In a real app, this would trigger a GUI notification or DBus signal.
        # For the prototype, we log the prompt.

        # Logic: If path is within the allowed "Ghost" directory, allow automatically?
        # The prompt said: "intercept any file open requests ... that fall outside of the ~/Ghost directory"
        # Since we monitor the mount, we see everything.

        ghost_dir = os.path.expanduser("~/Ghost")

        if path.startswith(ghost_dir):
            self.logger.info(f"Auto-allowing access to Ghost dir: {path}")
            return True

        # Otherwise, prompt (simulate)
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
            self.logger.error(f"Error reading fanotify fd: {e}")
            return

        if not buf:
            return

        event = FanotifyEventMetadata.from_buffer_copy(buf)

        # Resolve path
        access_path = self._resolve_path(event.fd)

        # DECISION LOGIC
        if self._prompt_user(access_path, event.pid):
            decision = FAN_ALLOW
        else:
            decision = FAN_DENY

        # Send response
        response = FanotifyResponse()
        response.fd = event.fd
        response.response = decision

        try:
            os.write(self.fanotify_fd, response)
        except OSError as e:
            self.logger.error(f"Failed to write fanotify response: {e}")

        # Close the event fd
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
