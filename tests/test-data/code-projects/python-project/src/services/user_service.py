"""User service module for the test Python project."""

from typing import List, Optional

from sqlalchemy.orm import Session

from .. import models
from ..main import UserCreate


def get_user(db: Session, user_id: int) -> Optional[models.User]:
    """Get user by ID."""
    return db.query(models.User).filter(models.User.id == user_id).first()


def get_user_by_email(db: Session, email: str) -> Optional[models.User]:
    """Get user by email."""
    return db.query(models.User).filter(models.User.email == email).first()


def get_users(db: Session, skip: int = 0, limit: int = 100) -> List[models.User]:
    """Get all users."""
    return db.query(models.User).offset(skip).limit(limit).all()


def create_user(db: Session, user: UserCreate) -> models.User:
    """Create a new user."""
    # TODO: Hash the password properly
    fake_hashed_password = user.password + "_hashed"
    db_user = models.User(
        email=user.email,
        name=user.name,
        hashed_password=fake_hashed_password,
    )
    db.add(db_user)
    db.commit()
    db.refresh(db_user)
    return db_user


def update_user(db: Session, user_id: int, user: UserCreate) -> Optional[models.User]:
    """Update a user."""
    db_user = get_user(db, user_id)
    if db_user:
        # TODO: Hash the password properly
        fake_hashed_password = user.password + "_hashed"
        db_user.email = user.email
        db_user.name = user.name
        db_user.hashed_password = fake_hashed_password
        db.commit()
        db.refresh(db_user)
    return db_user


def delete_user(db: Session, user_id: int) -> bool:
    """Delete a user."""
    db_user = get_user(db, user_id)
    if db_user:
        db.delete(db_user)
        db.commit()
        return True
    return False 