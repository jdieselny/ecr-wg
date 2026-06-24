import re

from rituals.runtime_paths import LOCAL_DENY_PATTERNS_PATH

class Sanitizer:
    BASELINE_PATTERNS = [
        r'\d{3}-\d{2}-\d{4}',          # SSN
        r'[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+', # Email
        r'(?<!\d)(?:\d[ -]*?){13,19}(?!\d)', # Credit card-like sequence
        r'(?<!\w)(?:\+?1[\s.-]?)?(?:\(?\d{3}\)?[\s.-]?)\d{3}[\s.-]?\d{4}(?!\w)', # Phone
    ]

    @classmethod
    def _local_patterns(cls):
        if not LOCAL_DENY_PATTERNS_PATH.exists():
            return []
        patterns = []
        for line in LOCAL_DENY_PATTERNS_PATH.read_text(encoding="utf-8").splitlines():
            stripped = line.strip()
            if stripped and not stripped.startswith("#"):
                patterns.append(stripped)
        return patterns

    @classmethod
    def _apply_pattern(cls, text, pattern):
        try:
            return re.sub(pattern, "[REDACTED]", text, flags=re.IGNORECASE)
        except re.error:
            return re.sub(re.escape(pattern), "[REDACTED]", text, flags=re.IGNORECASE)

    @classmethod
    def sanitize(cls, text):
        if text is None:
            return text
        if not isinstance(text, str):
            text = str(text)
        sanitized = text
        for pattern in cls.BASELINE_PATTERNS + cls._local_patterns():
            sanitized = cls._apply_pattern(sanitized, pattern)
        return sanitized
