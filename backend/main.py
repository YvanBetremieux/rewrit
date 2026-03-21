# rewrit backend API v1.0
from fastapi import FastAPI, Depends, HTTPException, Header
from fastapi.middleware.cors import CORSMiddleware
from sqlalchemy.orm import Session
from datetime import date, datetime, timedelta, timezone
from typing import Optional
import time

from database import engine, get_db, Base
from models import User, DailyQuota, UsageLog
from config import Config, get_mock_translation
from pydantic import BaseModel
import uuid as uuid_lib

# Create tables
Base.metadata.create_all(bind=engine)

app = FastAPI(title="rewrit API", version="1.0.0")

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=Config.ALLOWED_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

print("ℹ️  Backend manages quota and device registration")
print("   → AI translation is handled by Tauri frontend (Groq API)")


# Pydantic models
class QuotaInfo(BaseModel):
    used: int
    limit: int
    remaining: int
    resets_at: str


class RegisterRequest(BaseModel):
    platform: str
    app_version: str


class RegisterResponse(BaseModel):
    success: bool
    device_id: str
    quota: QuotaInfo


class QuotaResponse(BaseModel):
    quota: QuotaInfo
    is_pro: bool


class TranslateRequest(BaseModel):
    text: str


class TranslateResponse(BaseModel):
    success: bool
    original_text: Optional[str] = None
    translated_text: Optional[str] = None
    detected_language: Optional[str] = None
    target_language: Optional[str] = None
    processing_time_ms: Optional[int] = None
    quota: QuotaInfo
    error: Optional[str] = None
    message: Optional[str] = None


# Helper functions
def get_next_midnight_utc() -> datetime:
    """Get next midnight UTC"""
    now = datetime.now(timezone.utc)
    tomorrow = now + timedelta(days=1)
    next_midnight = datetime(tomorrow.year, tomorrow.month, tomorrow.day)
    return next_midnight


def get_or_create_user(db: Session, device_id: str) -> User:
    """Get or create user by device ID"""
    # Validate UUID format
    try:
        uuid_lib.UUID(device_id)
    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid device ID format")

    user = db.query(User).filter(User.device_id == device_id).first()

    if not user:
        user = User(device_id=device_id)
        db.add(user)
        db.commit()
        db.refresh(user)

    # Update last seen
    user.last_seen_at = datetime.now(timezone.utc)
    db.commit()

    return user


def get_or_create_daily_quota(db: Session, user_id: int, quota_date: date) -> DailyQuota:
    """Get or create daily quota for user"""
    quota = (
        db.query(DailyQuota)
        .filter(DailyQuota.user_id == user_id, DailyQuota.quota_date == quota_date)
        .first()
    )

    if not quota:
        quota = DailyQuota(
            user_id=user_id,
            quota_date=quota_date,
            actions_used=0,
            quota_limit=Config.DAILY_QUOTA_LIMIT
        )
        db.add(quota)
        db.commit()
        db.refresh(quota)

    return quota


def build_quota_info(quota: DailyQuota) -> QuotaInfo:
    """Build QuotaInfo from DailyQuota"""
    return QuotaInfo(
        used=quota.actions_used,
        limit=quota.quota_limit,
        remaining=max(0, quota.quota_limit - quota.actions_used),
        resets_at=get_next_midnight_utc().isoformat() + "Z",
    )


def check_and_decrement_quota(db: Session, user: User) -> tuple[bool, QuotaInfo]:
    """Check if user has quota and decrement if available"""
    today = date.today()
    quota = get_or_create_daily_quota(db, user.id, today)

    quota_info = build_quota_info(quota)

    # Check if quota exceeded
    if quota.actions_used >= quota.quota_limit:
        return False, quota_info

    # Increment usage
    quota.actions_used += 1
    db.commit()
    db.refresh(quota)

    # Return updated quota info
    updated_quota_info = build_quota_info(quota)
    return True, updated_quota_info


def translate_text_mock(text: str, source_lang: str, target_lang: str) -> str:
    """Mock translation for backend fallback (AI is handled by Tauri frontend via Groq)"""
    return get_mock_translation(text, source_lang, target_lang)


# API Endpoints
@app.get("/")
async def root():
    return {"message": "rewrit API v1.0.0"}


@app.get("/api/v1/status")
async def get_status():
    """Get API status and configuration"""
    return {
        "status": "running",
        "version": "1.0.0",
        "config": Config.get_status()
    }


@app.post("/api/v1/register", response_model=RegisterResponse)
async def register(
    request: RegisterRequest,
    x_device_id: str = Header(..., alias="X-Device-ID"),
    db: Session = Depends(get_db),
):
    """Register a new device"""
    user = get_or_create_user(db, x_device_id)
    today = date.today()
    quota = get_or_create_daily_quota(db, user.id, today)

    return RegisterResponse(
        success=True, device_id=str(user.device_id), quota=build_quota_info(quota)
    )


@app.get("/api/v1/quota", response_model=QuotaResponse)
async def get_quota(
    x_device_id: str = Header(..., alias="X-Device-ID"), db: Session = Depends(get_db)
):
    """Get quota information for a device"""
    user = get_or_create_user(db, x_device_id)
    today = date.today()
    quota = get_or_create_daily_quota(db, user.id, today)

    return QuotaResponse(quota=build_quota_info(quota), is_pro=user.is_pro)


@app.post("/api/v1/translate", response_model=TranslateResponse)
async def translate(
    request: TranslateRequest,
    x_device_id: str = Header(..., alias="X-Device-ID"),
    db: Session = Depends(get_db),
):
    """Translate text"""
    start_time = time.time()


    # Validate input
    if not request.text or len(request.text.strip()) == 0:
        raise HTTPException(status_code=400, detail="Text cannot be empty")

    if len(request.text) > 5000:
        raise HTTPException(status_code=400, detail="Text too long (max 5000 characters)")

    # Get user
    user = get_or_create_user(db, x_device_id)

    # Check and decrement quota
    quota_allowed, quota_info = check_and_decrement_quota(db, user)
    if not quota_allowed:
        raise HTTPException(status_code=403, detail="Quota exceeded")

    try:
        # Default to EN→FR (language direction is handled by Tauri frontend shortcuts)
        source_lang = "en"
        target_lang = "fr"

        # Mock translate (real AI translation is done client-side via Groq)
        translated_text = translate_text_mock(request.text, source_lang, target_lang)
        # Calculate processing time
        processing_time = int((time.time() - start_time) * 1000)

        # Log the action
        log_entry = UsageLog(
            user_id=user.id,
            action_type="translate",
            input_text_length=len(request.text),
            output_text_length=len(translated_text),
            processing_time_ms=processing_time,
            success=True,
        )
        db.add(log_entry)

        # Update total actions count
        user.total_actions_count += 1
        db.commit()
        return TranslateResponse(
            success=True,
            original_text=request.text,
            translated_text=translated_text,
            detected_language=source_lang,
            target_language=target_lang,
            processing_time_ms=processing_time,
            quota=quota_info,
        )

    except Exception as e:
        # Log failed action
        log_entry = UsageLog(
            user_id=user.id,
            action_type="translate",
            input_text_length=len(request.text),
            success=False,
            error_message=str(e),
        )
        db.add(log_entry)
        db.commit()

        return TranslateResponse(
            success=False,
            error="translation_failed",
            message=f"Service temporarily unavailable: {str(e)}",
            quota=quota_info,
        )


@app.post("/api/v1/usage/report")
async def report_usage(
    x_device_id: str = Header(..., alias="X-Device-ID"),
    db: Session = Depends(get_db),
):
    """Report a translation action performed by the Tauri frontend (Groq).
    Decrements quota and returns updated quota info."""
    user = get_or_create_user(db, x_device_id)

    quota_allowed, quota_info = check_and_decrement_quota(db, user)
    if not quota_allowed:
        raise HTTPException(status_code=403, detail="Quota exceeded")

    user.total_actions_count += 1
    db.commit()

    return {"success": True, "quota": quota_info}


@app.post("/api/v1/quota/reset")
async def reset_quota(
    x_device_id: str = Header(..., alias="X-Device-ID"),
    db: Session = Depends(get_db)
):
    """Reset quota to 0 for debugging purposes (only in development mode)"""

    # Only allow in development mode
    if Config.ENVIRONMENT != "development":
        print("⚠️  Quota reset: DISABLED (not in development mode)")
        raise HTTPException(status_code=403, detail="This endpoint is only available in development mode")

    # Get user (pass string, function will convert to UUID)
    user = get_or_create_user(db, x_device_id)
    print(user)

    # Get or create today's quota
    today = date.today()
    quota = get_or_create_daily_quota(db, user.id, today)

    # Reset to 0 and update quota_limit from config
    quota.actions_used = 0
    quota.quota_limit = Config.DAILY_QUOTA_LIMIT
    db.commit()
    db.refresh(quota)

    quota_info = build_quota_info(quota)

    return {
        "success": True,
        "message": "Quota reset to 0 (exhausted)",
        "quota": quota_info
    }


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        "main:app",  # Import string format for reload to work
        host="0.0.0.0",
        port=8000,
        reload=True,  # Enable hot reload for development
        reload_dirs=["./"]  # Watch current directory for changes
    )
