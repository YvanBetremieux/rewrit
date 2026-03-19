"""
Configuration and feature flags for rewrit backend
"""
import os
from dotenv import load_dotenv

load_dotenv()


class Config:
    """Application configuration"""

    # Database
    DATABASE_URL = os.getenv("DATABASE_URL", "postgresql://localhost/rewrit")

    # API Keys
    # ANTHROPIC_API_KEY = os.getenv("ANTHROPIC_API_KEY", "")  # Removed - now using Groq in frontend

    # Environment
    ENVIRONMENT = os.getenv("ENVIRONMENT", "development")
    PORT = int(os.getenv("PORT", 8000))

    # CORS
    ALLOWED_ORIGINS = os.getenv("ALLOWED_ORIGINS", "*").split(",")

    # Quota Configuration
    DAILY_QUOTA_LIMIT = int(os.getenv("DAILY_QUOTA_LIMIT", 5))

    # Rate Limiting
    RATE_LIMIT_PER_SECOND = int(os.getenv("RATE_LIMIT_PER_SECOND", 1))
    RATE_LIMIT_PER_HOUR = int(os.getenv("RATE_LIMIT_PER_HOUR", 100))

    # Feature Flags
    USE_AI = os.getenv("USE_AI", "false").lower() in ("true", "1", "yes")

    @classmethod
    def is_ai_enabled(cls) -> bool:
        """Check if AI translation is enabled (disabled - now using Groq in frontend)"""
        return False  # AI translation moved to Tauri frontend with Groq

    @classmethod
    def get_status(cls) -> dict:
        """Get current configuration status"""
        return {
            "environment": cls.ENVIRONMENT,
            "use_ai": False,  # AI translation moved to frontend
            "ai_enabled": False,
            "has_api_key": False,
            "database_configured": bool(cls.DATABASE_URL)
        }


# Hardcoded translations for testing (when USE_AI=false)
MOCK_TRANSLATIONS = {
    "en_to_fr": {
        "hello": "bonjour",
        "hello world": "bonjour le monde",
        "how are you": "comment allez-vous",
        "good morning": "bon matin",
        "thank you": "merci",
        "goodbye": "au revoir",
        "yes": "oui",
        "no": "non",
        "please": "s'il vous plaît",
        "welcome": "bienvenue",
    },
    "fr_to_en": {
        "bonjour": "hello",
        "bonjour le monde": "hello world",
        "comment allez-vous": "how are you",
        "bon matin": "good morning",
        "merci": "thank you",
        "au revoir": "goodbye",
        "oui": "yes",
        "non": "no",
        "s'il vous plaît": "please",
        "bienvenue": "welcome",
    }
}


def get_mock_translation(text: str, source_lang: str, target_lang: str) -> str:
    """Get a mock translation for testing"""
    key = f"{source_lang}_to_{target_lang}"
    translations = MOCK_TRANSLATIONS.get(key, {})

    # Try exact match (case-insensitive)
    text_lower = text.lower().strip()
    if text_lower in translations:
        # Preserve original case for first letter
        result = translations[text_lower]
        if text and text[0].isupper():
            result = result.capitalize()
        return result

    # Fallback: add [MOCK] prefix
    if source_lang == "en":
        return f"[MOCK FR] {text}"
    else:
        return f"[MOCK EN] {text}"
