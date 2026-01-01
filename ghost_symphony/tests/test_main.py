import unittest
from unittest.mock import patch, MagicMock, AsyncMock
import asyncio
import sys
# Update import to point to correct relative location if running as script,
# but for module run we rely on package structure.
from ghost_symphony.src.ghost_symphony.main import SessionOrchestrator, SecurityBreach

class TestSessionOrchestrator(unittest.TestCase):

    @patch("ghost_symphony.src.ghost_symphony.main.BaitIdentity")
    @patch("ghost_symphony.src.ghost_symphony.main.NetworkGhost")
    @patch("ghost_symphony.src.ghost_symphony.main.SandboxManager")
    @patch("ghost_symphony.src.ghost_symphony.main.Gatekeeper")
    def test_run_session_flow(self, MockGatekeeper, MockSandbox, MockNetwork, MockBait):
        orch = SessionOrchestrator()

        # Mock pre-flight checks to pass
        orch.pre_flight_checks = MagicMock()

        # Mock Sandbox async launch
        mock_process = AsyncMock(spec=asyncio.subprocess.Process)
        mock_process.pid = 9999
        mock_process.wait.return_value = None

        # Configure the mock instance that orch.sandbox uses
        orch.sandbox.launch_browser_async = AsyncMock(return_value=mock_process)

        # Run loop
        loop = asyncio.new_event_loop()
        orch.loop = loop
        loop.run_until_complete(orch.run_session())

        # Verify Sequence
        orch.pre_flight_checks.assert_called()
        orch.network.rotate_mac.assert_called()
        orch.sandbox.setup_audio_sink.assert_called()
        orch.bait.deploy_bait.assert_called()
        orch.gatekeeper.start_monitoring.assert_called()
        orch.sandbox.launch_browser_async.assert_called()

        # Verify Volatile Home Creation (mocked via tempfile? no we didn't mock tempfile in main)
        # We can check if launch_browser_async was called with a path
        call_args = orch.sandbox.launch_browser_async.call_args
        self.assertTrue(call_args[1]['real_home'].startswith('/tmp/'))

        loop.close()

    @patch("ghost_symphony.src.ghost_symphony.main.BaitIdentity")
    @patch("ghost_symphony.src.ghost_symphony.main.NetworkGhost")
    @patch("ghost_symphony.src.ghost_symphony.main.SandboxManager")
    @patch("ghost_symphony.src.ghost_symphony.main.Gatekeeper")
    def test_panic_on_breach(self, MockGatekeeper, MockSandbox, MockNetwork, MockBait):
        orch = SessionOrchestrator()
        orch.pre_flight_checks = MagicMock()

        # Simulate SecurityBreach during setup
        orch.sandbox.setup_audio_sink.side_effect = SecurityBreach("Fail")

        with self.assertRaises(SystemExit) as cm:
            loop = asyncio.new_event_loop()
            orch.loop = loop
            loop.run_until_complete(orch.run_session())
            loop.close()

        self.assertEqual(cm.exception.code, 1)

        orch.network.panic_shutdown.assert_called()

if __name__ == '__main__':
    unittest.main()
