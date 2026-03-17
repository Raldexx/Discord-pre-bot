from fastapi import FastAPI
from pydantic import BaseModel
from detector import ToxicityDetector
import uvicorn
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(title="Discord Bot AI Servisi", version="0.1.0")
detector = ToxicityDetector()


class AnalyzeRequest(BaseModel):
    text: str


class AnalyzeResponse(BaseModel):
    is_toxic: bool
    score: float
    reason: str


@app.on_event("startup")
async def startup():
    logger.info("AI servisi başlatılıyor...")
    detector.load()
    logger.info("Model yüklendi, servis hazır.")


@app.get("/health")
async def health():
    return {"status": "ok", "model_loaded": detector.is_loaded}


@app.post("/analyze", response_model=AnalyzeResponse)
async def analyze(req: AnalyzeRequest):
    if not req.text or len(req.text.strip()) < 2:
        return AnalyzeResponse(is_toxic=False, score=0.0, reason="")

    result = detector.analyze(req.text)
    return AnalyzeResponse(**result)


if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)
