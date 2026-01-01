import asyncio
import logging
import signal
import sys
import os

from ghost_symphony.src.ghost_symphony.mock_system import BaitIdentity
from ghost_symphony.src.ghost_symphony.network import NetworkGhost
from ghost_symphony.src.ghost_symphony.sandbox import SandboxManager
from ghost_symphony.src.ghost_symphony.gatekeeper import Gatekeeper

# Configure Logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("GhostSymphony")

class GhostOrchestrator:
    def __init__(self):
        self.bait = BaitIdentity(debug_mode=False)
        self.network = NetworkGhost()
        self.sandbox = SandboxManager()
        # Monitor user's home or specific sensitive dirs.
        # For demo, we monitor /tmp/ghost_monitor (dummy)
        self.monitor_path = os.path.expanduser("~")
        self.gatekeeper = Gatekeeper(self.monitor_path)
        self.loop = asyncio.get_event_loop()
        self.running = True

    def setup(self):
        """
        Initializes the environment.
        """
        logger.info("Initializing Ghost Symphony...")

        # 1. Deploy Bait
        ghost_ramdisk = "/tmp/ghost_symphony"
        self.bait.deploy_bait(ghost_ramdisk)

        # 2. Network Ghosting
        # Assuming wlan0 for demo, in real usage might be configurable or auto-detected
        self.network.rotate_mac("wlan0")
        self.network.harden_ttl()

        # 3. Audio Deception
        self.sandbox.setup_audio_sink()

        # 4. Gatekeeper
        self.gatekeeper.start_monitoring(self.loop)

    async def run_browser_session(self):
        """
        Launches the browser in the sandbox.
        """
        logger.info("Launching protected browser session...")
        # Example browser command
        browser_cmd = ["firefox"]

        # Run blocking subprocess in executor to avoid blocking the asyncio loop (and Gatekeeper)
        # Note: SandboxManager.run_browser uses subprocess.run (blocking).
        # We wrap it.
        await self.loop.run_in_executor(
            None,
            self.sandbox.run_browser,
            "/tmp/ghost_symphony",
            browser_cmd
        )

        logger.info("Browser session ended.")
        self.shutdown()

    def shutdown(self):
        """
        Clean up resources.
        """
        if not self.running:
            return
        self.running = False
        logger.info("Shutting down Ghost Symphony...")

        self.gatekeeper.stop_monitoring(self.loop)
        self.sandbox.cleanup_audio_sink()

        # We don't necessarily undo MAC rotation or TTL as those might be desired to persist,
        # but the kill-switch logic (panic_shutdown) is for emergencies.
        # Normal shutdown is graceful.

        # Stop loop
        for task in asyncio.all_tasks(self.loop):
            task.cancel()

        logger.info("Shutdown complete.")

    def panic(self):
        """
        Emergency kill switch.
        """
        logger.critical("PANIC SIGNAL RECEIVED!")
        self.network.panic_shutdown()
        self.shutdown()
        sys.exit(1)

def main():
    orchestrator = GhostOrchestrator()

    # Signal Handlers
    def handle_signal(sig, frame):
        if sig == signal.SIGINT or sig == signal.SIGTERM:
            logger.info("Signal received, shutting down...")
            orchestrator.shutdown()
            sys.exit(0)

    signal.signal(signal.SIGINT, handle_signal)
    signal.signal(signal.SIGTERM, handle_signal)

    try:
        orchestrator.setup()

        # Start browser task
        task = orchestrator.loop.create_task(orchestrator.run_browser_session())

        # Run loop
        orchestrator.loop.run_until_complete(task)

    except Exception as e:
        logger.critical(f"Unhandled exception: {e}")
        orchestrator.panic()

if __name__ == "__main__":
    main()
