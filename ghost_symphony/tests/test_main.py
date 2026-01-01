import unittest
from unittest.mock import patch, MagicMock
import asyncio
from ghost_symphony.src.ghost_symphony.main import GhostOrchestrator

class TestMainOrchestrator(unittest.TestCase):

    @patch("ghost_symphony.src.ghost_symphony.main.BaitIdentity")
    @patch("ghost_symphony.src.ghost_symphony.main.NetworkGhost")
    @patch("ghost_symphony.src.ghost_symphony.main.SandboxManager")
    @patch("ghost_symphony.src.ghost_symphony.main.Gatekeeper")
    def test_setup_flow(self, MockGatekeeper, MockSandbox, MockNetwork, MockBait):
        orch = GhostOrchestrator()
        orch.setup()

        # Verify Bait deployed
        orch.bait.deploy_bait.assert_called_with("/tmp/ghost_symphony")

        # Verify Network
        orch.network.rotate_mac.assert_called_with("wlan0")
        orch.network.harden_ttl.assert_called()

        # Verify Audio
        orch.sandbox.setup_audio_sink.assert_called()

        # Verify Gatekeeper
        orch.gatekeeper.start_monitoring.assert_called()

    @patch("ghost_symphony.src.ghost_symphony.main.BaitIdentity")
    @patch("ghost_symphony.src.ghost_symphony.main.NetworkGhost")
    @patch("ghost_symphony.src.ghost_symphony.main.SandboxManager")
    @patch("ghost_symphony.src.ghost_symphony.main.Gatekeeper")
    def test_shutdown(self, MockGatekeeper, MockSandbox, MockNetwork, MockBait):
        orch = GhostOrchestrator()
        orch.setup() # Initialize mocks
        orch.shutdown()

        orch.sandbox.cleanup_audio_sink.assert_called()
        orch.gatekeeper.stop_monitoring.assert_called()

    @patch("ghost_symphony.src.ghost_symphony.main.BaitIdentity")
    @patch("ghost_symphony.src.ghost_symphony.main.NetworkGhost")
    @patch("ghost_symphony.src.ghost_symphony.main.SandboxManager")
    @patch("ghost_symphony.src.ghost_symphony.main.Gatekeeper")
    def test_panic(self, MockGatekeeper, MockSandbox, MockNetwork, MockBait):
        orch = GhostOrchestrator()
        with self.assertRaises(SystemExit):
            orch.panic()

        orch.network.panic_shutdown.assert_called()

if __name__ == '__main__':
    unittest.main()
