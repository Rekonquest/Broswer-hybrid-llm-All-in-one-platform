import os
import subprocess
import logging
import shlex

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
        # pactl load-module module-null-sink sink_name=Ghost_Mic media.class=Audio/Source/Virtual node.description="Integrated Microphone"
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
             # It might have been already unloaded or never loaded.
             pass
        except FileNotFoundError:
             pass

    def build_bwrap_command(self, target_dir: str, browser_command: list) -> list:
        """
        Constructs the bubblewrap command.

        Args:
            target_dir: The directory where the bait files (cpuinfo, meminfo) are stored.
            browser_command: The command to launch the browser (e.g. ["firefox"]).
        """
        cpu_path = os.path.join(target_dir, "cpuinfo")
        mem_path = os.path.join(target_dir, "meminfo")
        # Assuming target_dir is the ramdisk path e.g. /tmp/ghost_symphony or similar.
        # The prompt says: "--bind /tmp/ghost_symphony/cpuinfo /proc/cpuinfo"

        # We also need a place for the home directory.
        # The prompt says: "--tmpfs /home/user" and "--bind ~/Ghost /home/user"
        # Since we are running as the current user, we probably want to map the current user's home
        # or a fake home. The prompt says "bind-mount fake ... /proc/cpuinfo".
        # The prompt specific command:
        # bwrap --ro-bind / / \
        #       --dev-bind /dev /dev \
        #       --proc /proc \
        #       --tmpfs /home/user \
        #       --bind ~/Ghost /home/user \
        #       --bind /tmp/ghost_symphony/cpuinfo /proc/cpuinfo \
        #       --bind /tmp/ghost_symphony/meminfo /proc/meminfo \
        #       --setenv LIBGL_ALWAYS_SOFTWARE 1 \
        #       --unshare-all \
        #       --hostname ghost-station \
        #       [BROWSER_COMMAND]

        # We need to resolve ~/Ghost to an absolute path
        ghost_home = os.path.expanduser("~/Ghost")

        # Ensure ghost_home exists
        if not os.path.exists(ghost_home):
            os.makedirs(ghost_home, exist_ok=True)

        cmd = [
            "bwrap",
            "--ro-bind", "/", "/",
            "--dev-bind", "/dev", "/dev",
            "--proc", "/proc",
            # We need to handle the home directory carefully.
            # If we map /home/user, we need to know the current user's home or target user.
            # For simplicity, let's assume we map the current user's home to the tmpfs/ghost bind.
            # But the prompt says "--tmpfs /home/user". This implies we are creating a new home.
            # Let's use the actual user's home path if possible, or just follow the prompt literally if "user" is a placeholder.
            # "I am the only user on this profile" -> So likely /home/<username>.

            # Let's dynamically determine the home mount point
            # "--tmpfs", os.path.expanduser("~"),
            # Actually, bubblewrap is tricky. If we mask the real home, we mask X11/Wayland sockets too usually?
            # But let's stick to the prompt's suggested structure.
        ]

        user_home = os.path.expanduser("~")

        cmd.extend([
            "--tmpfs", user_home,
            "--bind", ghost_home, user_home,
            "--bind", cpu_path, "/proc/cpuinfo",
            "--bind", mem_path, "/proc/meminfo",
            "--setenv", "LIBGL_ALWAYS_SOFTWARE", "1",
            "--unshare-all",
            "--hostname", "ghost-station"
        ])

        # Add browser command
        cmd.extend(browser_command)

        return cmd

    def run_browser(self, target_dir: str, browser_command: list):
        """
        Runs the browser inside the sandbox.
        """
        cmd = self.build_bwrap_command(target_dir, browser_command)
        self.logger.info(f"Launching sandbox with command: {shlex.join(cmd)}")

        try:
            # We use subprocess.Popen so we don't block? Or run?
            # If main orchestrator waits for it, run is fine.
            # But usually we want to run it and wait.
            subprocess.run(cmd, check=True)
        except subprocess.CalledProcessError as e:
            self.logger.error(f"Sandbox exited with error: {e}")
        except FileNotFoundError:
            self.logger.error("bwrap not found. Is Bubblewrap installed?")
