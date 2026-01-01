import unittest
from unittest.mock import MagicMock, patch
import asyncio
import os
from ctypes import CDLL
from ghost_symphony.src.ghost_symphony.gatekeeper import Gatekeeper, FanotifyEventMetadata, FanotifyResponse

class TestGatekeeper(unittest.TestCase):
    def setUp(self):
        self.gatekeeper = Gatekeeper("/tmp")

    def test_ctypes_structs(self):
        # Verify sizes and fields roughly
        event = FanotifyEventMetadata()
        self.assertEqual(event.event_len, 0)
        resp = FanotifyResponse()
        self.assertEqual(resp.response, 0)

    @patch("ghost_symphony.src.ghost_symphony.gatekeeper.CDLL")
    def test_start_monitoring_success(self, mock_cdll):
        # Mock libc
        mock_libc = MagicMock()
        mock_cdll.return_value = mock_libc
        mock_libc.fanotify_init.return_value = 10 # Fake FD
        mock_libc.fanotify_mark.return_value = 0 # Success

        loop = MagicMock()

        self.gatekeeper.start_monitoring(loop)

        self.assertEqual(self.gatekeeper.fanotify_fd, 10)
        mock_libc.fanotify_init.assert_called()
        mock_libc.fanotify_mark.assert_called()
        loop.add_reader.assert_called_with(10, self.gatekeeper.handle_event)

    @patch("ghost_symphony.src.ghost_symphony.gatekeeper.CDLL")
    def test_start_monitoring_fail_init(self, mock_cdll):
        mock_libc = MagicMock()
        mock_cdll.return_value = mock_libc
        mock_libc.fanotify_init.return_value = -1 # Failure

        loop = MagicMock()
        self.gatekeeper.start_monitoring(loop)

        # Should log error and return
        self.assertEqual(self.gatekeeper.fanotify_fd, -1)
        loop.add_reader.assert_not_called()

if __name__ == '__main__':
    unittest.main()
