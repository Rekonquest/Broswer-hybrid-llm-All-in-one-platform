import unittest
from unittest.mock import patch, MagicMock, AsyncMock
import asyncio
import os
from ghost_symphony.src.ghost_symphony.sandbox import SandboxManager

class TestSandboxManager(unittest.TestCase):
    def setUp(self):
        self.sandbox = SandboxManager()

    @patch("subprocess.run")
    def test_setup_audio_sink(self, mock_run):
        self.sandbox.setup_audio_sink()
        mock_run.assert_called_with(
            ['pactl', 'load-module', 'module-null-sink',
             'sink_name=Ghost_Mic',
             'media.class=Audio/Source/Virtual',
             'node.description="Integrated Microphone"'],
            check=True, capture_output=True
        )

    def test_build_bwrap_command(self):
        target_dir = "/tmp/fake_sys"
        real_home = "/tmp/volatile_home"
        browser_cmd = ["firefox"]
        cmd = self.sandbox.build_bwrap_command(target_dir, browser_cmd, real_home)

        self.assertIn("bwrap", cmd)
        self.assertIn("--ro-bind", cmd)

        # Check specific bindings
        self.assertIn(os.path.join(target_dir, "cpuinfo"), cmd)

        # Check Home binding
        self.assertIn(real_home, cmd)

        # Check Isolation flags (ensure unshare-net is NOT present)
        self.assertIn("--unshare-pid", cmd)
        self.assertNotIn("--unshare-net", cmd)

    @patch("asyncio.create_subprocess_exec", new_callable=AsyncMock)
    def test_launch_browser_async(self, mock_exec):
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)

        target_dir = "/tmp/fake_sys"
        real_home = "/tmp/volatile_home"
        browser_cmd = ["firefox"]

        # Mock the process object
        mock_process = MagicMock()
        mock_process.pid = 12345
        mock_exec.return_value = mock_process

        process = loop.run_until_complete(
            self.sandbox.launch_browser_async(target_dir, browser_cmd, real_home)
        )

        self.assertEqual(process.pid, 12345)
        mock_exec.assert_called()

        # Check args passed to exec
        call_args = mock_exec.call_args
        self.assertIn("bwrap", call_args[0])
        loop.close()

if __name__ == '__main__':
    unittest.main()
