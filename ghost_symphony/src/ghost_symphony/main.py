import asyncio
import logging
import signal
import sys
import os
import shutil
import subprocess
import tempfile

from .mock_system import BaitIdentity
from .network import NetworkGhost
from .sandbox import SandboxManager
from .gatekeeper import Gatekeeper, SecurityBreach

# Configure Logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("GhostSymphony")

class SessionOrchestrator:
    def __init__(self):
        self.bait = BaitIdentity(debug_mode=False)
        self.network = NetworkGhost()
        self.sandbox = SandboxManager()
        self.monitor_path = os.path.expanduser("~")
        self.gatekeeper = Gatekeeper(self.monitor_path)
        self.loop = asyncio.get_event_loop()
        self.running = True
        self.browser_process = None
        self.ghost_ramdisk = "/tmp/ghost_symphony" # Hardware bait dir
        self.ghost_home = None # Volatile home dir

    def pre_flight_checks(self):
        """
        Verifies system readiness before launch.
        """
        logger.info("Running Pre-Flight Checks...")
        if not os.access("/tmp", os.W_OK):
            raise RuntimeError("Cannot write to /tmp for RAM-disk.")
        logger.info("Pre-Flight Checks Passed.")

    def setup_environment(self):
        """
        Executes the strict startup sequence.
        """
        logger.info("Orchestrating Ghost Session Environment...")

        # 1. Network Setup
        self.network.rotate_mac("wlan0")
        self.network.harden_ttl()

        # 2. Audio/Video
        self.sandbox.setup_audio_sink()

        # 3. Bait Deployment
        self.bait.deploy_bait(self.ghost_ramdisk)

        # 4. Volatile Home (RAM-Disk simulation)
        # We create a temp dir which is effectively on tmpfs usually.
        self.ghost_home = tempfile.mkdtemp(prefix="ghost_home_")
        logger.info(f"Created volatile home at {self.ghost_home}")

        # 5. Gatekeeper
        self.gatekeeper.start_monitoring(self.loop)

    async def run_session(self):
        """
        Launches the browser and manages the session lifecycle.
        """
        try:
            self.pre_flight_checks()
            self.setup_environment()

            logger.info("Launching protected browser session (Async)...")
            browser_cmd = ["firefox"]

            # Pass the volatile home to the sandbox manager
            self.browser_process = await self.sandbox.launch_browser_async(
                self.ghost_ramdisk,
                browser_cmd,
                real_home=self.ghost_home
            )

            if not self.browser_process:
                raise RuntimeError("Failed to launch browser process.")

            logger.info(f"Targeting Gatekeeper on PID: {self.browser_process.pid}")
            self.gatekeeper.set_target_pid(self.browser_process.pid)

            await self.browser_process.wait()

            logger.info("Browser session ended normally.")
            self.shutdown()

        except SecurityBreach as e:
            logger.critical(f"SECURITY BREACH DETECTED: {e}")
            self.panic()
        except Exception as e:
            logger.critical(f"Session Error: {e}")
            self.panic()

    def shutdown(self):
        """
        Graceful cleanup.
        """
        if not self.running:
            return
        self.running = False
        logger.info("Shutting down Ghost Symphony...")

        self.gatekeeper.stop_monitoring(self.loop)
        self.sandbox.cleanup_audio_sink()

        # Cleanup Hardware Bait
        if os.path.exists(self.ghost_ramdisk):
            shutil.rmtree(self.ghost_ramdisk, ignore_errors=True)

        # Cleanup Volatile Home (Data Wipe)
        if self.ghost_home and os.path.exists(self.ghost_home):
            shutil.rmtree(self.ghost_home, ignore_errors=True)
            logger.info("Wiped Volatile Home (RAM-disk).")

        current_task = asyncio.current_task(self.loop)
        for task in asyncio.all_tasks(self.loop):
            if task is not current_task:
                task.cancel()

        logger.info("Shutdown complete.")

    def panic(self):
        """
        Kill-Switch: Hard kill browser process group and network.
        """
        logger.critical("!!! INITIATING PANIC SHUTDOWN !!!")

        self.network.panic_shutdown()

        if self.browser_process:
            try:
                pgid = os.getpgid(self.browser_process.pid)
                os.killpg(pgid, signal.SIGKILL)
                logger.critical(f"Killed browser process group: {pgid}")
            except (ProcessLookupError, OSError):
                try:
                    self.browser_process.kill()
                except Exception:
                    pass

        try:
             subprocess.run(["sysctl", "-w", "net.ipv4.ip_default_ttl=64"], check=False, capture_output=True)
        except Exception:
             pass

        self.shutdown()
        sys.exit(1)

def main():
    orchestrator = SessionOrchestrator()

    def handle_signal(sig, frame):
        if sig == signal.SIGINT or sig == signal.SIGTERM:
            logger.info("Signal received, stopping session...")
            orchestrator.shutdown()
            sys.exit(0)

    signal.signal(signal.SIGINT, handle_signal)
    signal.signal(signal.SIGTERM, handle_signal)

    try:
        task = orchestrator.loop.create_task(orchestrator.run_session())
        orchestrator.loop.run_until_complete(task)
    except (KeyboardInterrupt, SystemExit):
        pass
    except Exception as e:
        logger.critical(f"Unhandled Orchestrator Exception: {e}")
        orchestrator.panic()

if __name__ == "__main__":
    main()
