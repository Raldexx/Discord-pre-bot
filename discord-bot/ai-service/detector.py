import re
import logging
from typing import Optional

logger = logging.getLogger(__name__)

# Türkçe küfür/hakaret pattern'ları (şifreli versiyonlar dahil)
TURKISH_TOXIC_PATTERNS = [
    # Küfürler - doğrudan
    r"\boç\b", r"\bamu[kq]\b", r"\bsik\b", r"\bs[1!]k\b",
    r"\borospu\b", r"\bpiç\b", r"\bp[1!]ç\b", r"\bboktan\b",
    r"\bgerizekal[iı]\b", r"\baptal\b", r"\bsalak\b", r"\bnobody\b",
    r"\bkafas[iı]z\b", r"\byavşak\b", r"\ballahsız\b",

    # Şifreli versiyonlar (filtre atlatma girişimi)
    r"s[\*\.\-_]k", r"o[\*\.\-_]ç", r"p[\*\.\-_]ç",
    r"[a4]m[u0]k", r"or[o0]spu",

    # Tehdit içerikli
    r"(seni|sizi).{0,10}(öldür|gebertir|bitir)",
    r"(ip[e]|idam).{0,5}(çek|as)",
    r"kafan[ıi].{0,5}(patla|kır)",
]

# İngilizce toxic pattern'lar (temel)
ENGLISH_TOXIC_PATTERNS = [
    r"\bf[u*]ck\b", r"\bs[h*]it\b", r"\bb[i*]tch\b",
    r"\bk[i*]ll\s+your?self\b", r"\bdie\s+already\b",
    r"\bn[i*]gg[ae]r\b", r"\bfagg[o0]t\b",
]

# Ağır tehdit içeren pattern'lar (anında aksiyon)
SEVERE_PATTERNS = [
    r"(kendini|kendinizi).{0,10}(öldür|as|zehirle)",
    r"kill\s+your?self",
    r"kys\b",
    r"(bomb|bomba).{0,10}(at|patlat|yerleştir)",
]

# Kavga/gerilim tespiti için kelimeler
AGGRESSION_KEYWORDS = [
    "seni tanıdım", "bulacağım", "pişman olacaksın", "hesap soracağım",
    "gözünü sikerim", "bekle", "adresini", "ip adres",
    "you're dead", "gonna find you", "watch your back",
]


class ToxicityDetector:
    def __init__(self):
        self.is_loaded = False
        self._detoxify_model = None
        self._turkish_patterns = None
        self._english_patterns = None
        self._severe_patterns = None

    def load(self):
        """Modeli ve pattern'ları yükle."""
        # Pattern'ları derle
        self._turkish_patterns = [
            re.compile(p, re.IGNORECASE | re.UNICODE)
            for p in TURKISH_TOXIC_PATTERNS
        ]
        self._english_patterns = [
            re.compile(p, re.IGNORECASE)
            for p in ENGLISH_TOXIC_PATTERNS
        ]
        self._severe_patterns = [
            re.compile(p, re.IGNORECASE | re.UNICODE)
            for p in SEVERE_PATTERNS
        ]

        # Detoxify modelini yüklemeyi dene (opsiyonel, yoksa pattern'larla devam et)
        try:
            from detoxify import Detoxify
            self._detoxify_model = Detoxify("multilingual")
            logger.info("Detoxify modeli yüklendi.")
        except ImportError:
            logger.warning("Detoxify bulunamadı, yalnızca pattern tabanlı analiz kullanılacak.")
        except Exception as e:
            logger.warning(f"Detoxify yüklenemedi: {e}. Pattern tabanlı analiz aktif.")

        self.is_loaded = True

    def analyze(self, text: str) -> dict:
        """
        Metni analiz et ve toksisite sonucu döndür.
        Returns: {is_toxic, score, reason}
        """
        if not self.is_loaded:
            return {"is_toxic": False, "score": 0.0, "reason": ""}

        text_clean = text.strip()

        # 1. Ağır tehdit kontrolü (en öncelikli)
        for pattern in self._severe_patterns:
            if pattern.search(text_clean):
                logger.info(f"Ağır tehdit tespit edildi: {text_clean[:50]}")
                return {
                    "is_toxic": True,
                    "score": 1.0,
                    "reason": "Ağır tehdit içeriği",
                }

        # 2. Türkçe pattern kontrolü
        for pattern in self._turkish_patterns:
            if pattern.search(text_clean):
                return {
                    "is_toxic": True,
                    "score": 0.9,
                    "reason": "Küfür/hakaret içeriği",
                }

        # 3. İngilizce pattern kontrolü
        for pattern in self._english_patterns:
            if pattern.search(text_clean):
                return {
                    "is_toxic": True,
                    "score": 0.85,
                    "reason": "Offensive language",
                }

        # 4. Saldırganlık kelime kontrolü
        text_lower = text_clean.lower()
        for keyword in AGGRESSION_KEYWORDS:
            if keyword in text_lower:
                return {
                    "is_toxic": True,
                    "score": 0.75,
                    "reason": "Tehdit/saldırgan dil",
                }

        # 5. Detoxify ML modeli (varsa)
        if self._detoxify_model:
            try:
                results = self._detoxify_model.predict(text_clean)
                toxicity_score = float(results.get("toxicity", 0))
                severe_score = float(results.get("severe_toxicity", 0))
                threat_score = float(results.get("threat", 0))

                max_score = max(toxicity_score, severe_score * 1.5, threat_score * 1.3)

                if max_score > 0.75:
                    reason = self._get_reason(results)
                    return {
                        "is_toxic": True,
                        "score": round(max_score, 3),
                        "reason": reason,
                    }
            except Exception as e:
                logger.error(f"Detoxify analiz hatası: {e}")

        return {"is_toxic": False, "score": 0.0, "reason": ""}

    def _get_reason(self, results: dict) -> str:
        """Detoxify sonuçlarından neden üret."""
        labels = {
            "toxicity": "Toksik içerik",
            "severe_toxicity": "Ağır toksik içerik",
            "obscene": "Müstehcen içerik",
            "threat": "Tehdit içeriği",
            "insult": "Hakaret içeriği",
            "identity_attack": "Kimliğe saldırı",
        }
        top = max(results, key=lambda k: results.get(k, 0))
        return labels.get(top, "Uygunsuz içerik")
