import re
import logging

logger = logging.getLogger(__name__)

# ----------------------------------------------------------------
# TOXIC PATTERNS — English + Global
# ----------------------------------------------------------------

OFFENSIVE_PATTERNS = [
    r"\bf[u*@]ck\b", r"\bs[h*]it\b", r"\bb[i*]tch\b", r"\ba[s*]{2}hole\b",
    r"\bc[u*]nt\b", r"\bd[i*]ck\b", r"\bp[u*]ssy\b", r"\bwh[o*]re\b",
    r"\bn[i*]gg[ae]r\b", r"\bf[a*]gg[o0]t\b", r"\br[e*]tard\b",
    r"\bk[i*]ke\b", r"\bsp[i*]c\b", r"\bch[i*]nk\b",
    # Cipher bypass attempts
    r"f[\*\.\-_\s]u[\*\.\-_\s]c[\*\.\-_\s]k",
    r"s[\*\.\-_\s]h[\*\.\-_\s]i[\*\.\-_\s]t",
    r"[a4]s+h+[o0]+l+[e3]+",
]

THREAT_PATTERNS = [
    r"(i('ll|will|am going to|gonna)).{0,20}(kill|murder|hurt|destroy)\s+you",
    r"(you('re| are)).{0,10}(dead|finished|done)",
    r"(find|track|locate).{0,15}(you|your (house|address|ip|location))",
    r"watch\s+your\s+back",
    r"(gonna|going to).{0,10}(get|hurt|kill)\s+you",
]

SEVERE_PATTERNS = [
    r"kill\s+your\s*self",
    r"\bkys\b",
    r"go\s+die",
    r"end\s+your\s*(life|self)",
    r"(you\s+should|just).{0,10}(die|kill yourself|end it)",
    r"(bomb|shoot|attack).{0,15}(school|place|everyone|server)",
]

AGGRESSION_KEYWORDS = [
    "i know where you live", "come find me", "say that to my face",
    "you're dead", "dead man", "catch these hands", "on sight",
    "pulling up", "run your location", "drop your addy",
    "i'll end you", "say it again i dare you",
]

SPAM_TOXIC_PATTERNS = [
    r"(raid|nuke).{0,10}(this|the)\s+server",
    r"everyone\s+(leave|quit|raid)",
    r"@everyone.{0,20}(raid|attack|spam)",
]


class ToxicityDetector:
    def __init__(self):
        self.is_loaded = False
        self._detoxify_model = None
        self._compiled = {}

    def load(self):
        self._compiled["offensive"]  = [re.compile(p, re.IGNORECASE) for p in OFFENSIVE_PATTERNS]
        self._compiled["threat"]     = [re.compile(p, re.IGNORECASE) for p in THREAT_PATTERNS]
        self._compiled["severe"]     = [re.compile(p, re.IGNORECASE) for p in SEVERE_PATTERNS]
        self._compiled["spam_toxic"] = [re.compile(p, re.IGNORECASE) for p in SPAM_TOXIC_PATTERNS]

        try:
            from detoxify import Detoxify
            self._detoxify_model = Detoxify("original")
            logger.info("Detoxify model loaded.")
        except Exception as e:
            logger.warning(f"Detoxify unavailable: {e}. Pattern-based analysis only.")

        self.is_loaded = True
        logger.info("ToxicityDetector ready.")

    def analyze(self, text: str) -> dict:
        if not self.is_loaded or len(text.strip()) < 2:
            return {"is_toxic": False, "score": 0.0, "reason": "", "category": "clean"}

        t = text.strip()

        # Priority 1 — Severe
        for p in self._compiled["severe"]:
            if p.search(t):
                return {"is_toxic": True, "score": 1.0,  "reason": "Severe harmful content",       "category": "severe"}

        # Priority 2 — Threats
        for p in self._compiled["threat"]:
            if p.search(t):
                return {"is_toxic": True, "score": 0.9,  "reason": "Threatening language",          "category": "threat"}

        # Priority 3 — Offensive / slurs
        for p in self._compiled["offensive"]:
            if p.search(t):
                return {"is_toxic": True, "score": 0.85, "reason": "Offensive language or slur",    "category": "offensive"}

        # Priority 4 — Raid incitement
        for p in self._compiled["spam_toxic"]:
            if p.search(t):
                return {"is_toxic": True, "score": 0.8,  "reason": "Server raid incitement",        "category": "raid_incite"}

        # Priority 5 — Aggression keywords
        t_lower = t.lower()
        for kw in AGGRESSION_KEYWORDS:
            if kw in t_lower:
                return {"is_toxic": True, "score": 0.75, "reason": "Aggressive / threatening tone", "category": "aggression"}

        # Priority 6 — Detoxify ML
        if self._detoxify_model:
            try:
                r = self._detoxify_model.predict(t)
                score = max(
                    float(r.get("toxicity",        0)),
                    float(r.get("severe_toxicity", 0)) * 1.5,
                    float(r.get("threat",          0)) * 1.4,
                    float(r.get("insult",          0)) * 1.1,
                    float(r.get("identity_attack", 0)) * 1.3,
                )
                if score > 0.75:
                    top = max(r, key=lambda k: r.get(k, 0))
                    labels = {
                        "toxicity":        ("Toxic content",          "toxic"),
                        "severe_toxicity": ("Severely toxic content", "severe"),
                        "obscene":         ("Obscene content",        "obscene"),
                        "threat":          ("Threatening content",    "threat"),
                        "insult":          ("Insulting content",      "insult"),
                        "identity_attack": ("Identity-based attack",  "identity"),
                    }
                    reason, cat = labels.get(top, ("Inappropriate content", "other"))
                    return {"is_toxic": True, "score": round(min(score, 1.0), 3), "reason": reason, "category": cat}
            except Exception as e:
                logger.error(f"Detoxify error: {e}")

        return {"is_toxic": False, "score": 0.0, "reason": "", "category": "clean"}
