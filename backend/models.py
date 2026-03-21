from sqlalchemy import Boolean, Column, Integer, String, DateTime, Date, ForeignKey, Text, UniqueConstraint
from sqlalchemy.sql import func
from sqlalchemy.orm import relationship
from database import Base
import uuid


class User(Base):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True, index=True)
    device_id = Column(String(36), unique=True, nullable=False, default=lambda: str(uuid.uuid4()))
    created_at = Column(DateTime(timezone=True), server_default=func.now())
    last_seen_at = Column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())
    total_actions_count = Column(Integer, default=0)
    is_pro = Column(Boolean, default=False)

    # Relationships
    usage_logs = relationship("UsageLog", back_populates="user")
    daily_quotas = relationship("DailyQuota", back_populates="user")


class UsageLog(Base):
    __tablename__ = "usage_logs"

    id = Column(Integer, primary_key=True, index=True)
    user_id = Column(Integer, ForeignKey("users.id"), nullable=False)
    action_type = Column(String(50), nullable=False)  # 'translate'
    input_text_length = Column(Integer)
    output_text_length = Column(Integer)
    processing_time_ms = Column(Integer)
    success = Column(Boolean)
    error_message = Column(Text)
    created_at = Column(DateTime(timezone=True), server_default=func.now())

    # Relationship
    user = relationship("User", back_populates="usage_logs")


class DailyQuota(Base):
    __tablename__ = "daily_quotas"

    id = Column(Integer, primary_key=True, index=True)
    user_id = Column(Integer, ForeignKey("users.id"), nullable=False)
    quota_date = Column(Date, nullable=False)
    actions_used = Column(Integer, default=0)
    quota_limit = Column(Integer, default=5)

    __table_args__ = (UniqueConstraint('user_id', 'quota_date', name='uq_user_quota_date'),)

    # Relationship
    user = relationship("User", back_populates="daily_quotas")
