import os
import subprocess
import logging
import shlex
import asyncio

class SandboxManager:
    """
    Manages the Bubblewrap execution and PipeWire virtual sink.
    """

    def __init__(self):
        self.logger = logging.getLogger("SandboxManager")
        self.sink_name = "Ghost_Mic"

    def setup_audio_sink(self):
        """
        Creates a null sink to redirect audio to a void.
        """
        self.logger.info("Setting up audio deception (Null Sink)...")
        cmd = [
            "pactl", "load-module", "module-null-sink",
            f"sink_name={self.sink_name}",
            "media.class=Audio/Source/Virtual",
            'node.description="Integrated Microphone"'
        ]
        try:
            subprocess.run(cmd, check=True, capture_output=True)
            self.logger.info(f"Created Null Sink: {self.sink_name}")
        except subprocess.CalledProcessError as e:
            self.logger.error(f"Failed to create audio sink: {e}")
        except FileNotFoundError:
             self.logger.error("pactl not found. Is PulseAudio/PipeWire installed?")

    def cleanup_audio_sink(self):
        """
        Removes the null sink.
        """
        self.logger.info("Cleaning up audio sink...")
        cmd = ["pactl", "unload-module", "module-null-sink"]
        try:
            subprocess.run(cmd, check=True, capture_output=True)
            self.logger.info("Unloaded Null Sink module")
        except subprocess.CalledProcessError:
             pass
        except FileNotFoundError:
             pass

    def build_bwrap_command(self, target_dir: str, browser_command: list, real_home: str) -> list:
        """
        Constructs the bubblewrap command.

        Args:
            target_dir: The directory where the bait files (cpuinfo, meminfo) are stored.
            browser_command: The command to launch the browser (e.g. ["firefox"]).
            real_home: The temporary directory on the host to be bound as /home/user.
        """
        cpu_path = os.path.join(target_dir, "cpuinfo")
        mem_path = os.path.join(target_dir, "meminfo")

        # User in the sandbox
        sandbox_user_home = os.path.expanduser("~") # e.g. /home/jules

        cmd = [
            "bwrap",
            "--ro-bind", "/", "/",
            "--dev-bind", "/dev", "/dev",
            "--proc", "/proc",

            # Map the provided volatile home to the sandbox user's home
            "--bind", real_home, sandbox_user_home,

            # Map Bait
            "--bind", cpu_path, "/proc/cpuinfo",
            "--bind", mem_path, "/proc/meminfo",

            # Environment
            "--setenv", "LIBGL_ALWAYS_SOFTWARE", "1",

            # Isolation
            # We explicitly list namespaces to exclude network (so we share net)
            "--unshare-user",
            "--unshare-ipc",
            "--unshare-pid",
            "--unshare-uts",
            "--unshare-cgroup",
            # "--unshare-net", # Omitted to allow internet access

            "--hostname", "ghost-station"
        ]

        # Add browser command
        cmd.extend(browser_command)

        return cmd

    async def launch_browser_async(self, target_dir: str, browser_command: list, real_home: str) -> asyncio.subprocess.Process:
        """
        Launches the browser inside the sandbox asynchronously.
        Returns the asyncio Process object.
        """
        cmd = self.build_bwrap_command(target_dir, browser_command, real_home)
        self.logger.info(f"Launching sandbox asynchronously: {shlex.join(cmd)}")

        try:
            process = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            self.logger.info(f"Sandbox launched with PID: {process.pid}")
            return process
        except FileNotFoundError:
            self.logger.error("bwrap not found. Is Bubblewrap installed?")
            raise
