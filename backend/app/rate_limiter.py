import time
from collections import deque
from typing import Deque, Dict

# メモリ上でIPアドレスを記録してレートリミットを行う
class SlidingWindowRateLimiter:
    def __init__(self, max_requests: int, window_seconds: int) -> None:
        self.max_requests = max_requests
        self.window_seconds = window_seconds
        self._buckets: Dict[str, Deque[float]] = {}

    def allow(self, key: str) -> bool:
        now = time.time()
        dq = self._buckets.get(key)
        if dq is None:
            dq = deque()
            self._buckets[key] = dq
        threshold = now - self.window_seconds
        while dq and dq[0] < threshold:
            dq.popleft()
        if len(dq) >= self.max_requests:
            return False
        dq.append(now)
        return True
