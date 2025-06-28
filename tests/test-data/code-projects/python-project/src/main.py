"""Main module for the test Python project."""

import logging
import sys
from typing import List, Optional

import uvicorn
from fastapi import Depends, FastAPI, HTTPException, Query
from pydantic import BaseModel
from sqlalchemy.orm import Session

from .database import SessionLocal, engine
from .models import Base
from .services import user_service

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger(__name__)

# Create database tables
Base.metadata.create_all(bind=engine)

# Create FastAPI app
app = FastAPI(
    title="Test Python Project",
    description="A mock Python project for testing",
    version="0.1.0",
)


# Dependency to get database session
def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()


# Models
class UserBase(BaseModel):
    email: str
    name: str


class UserCreate(UserBase):
    password: str


class User(UserBase):
    id: int
    is_active: bool

    class Config:
        orm_mode = True


# Routes
@app.get("/")
def read_root():
    """Root endpoint."""
    return {"message": "Welcome to Test Python Project API"}


@app.get("/users/", response_model=List[User])
def read_users(
    skip: int = 0,
    limit: int = 100,
    db: Session = Depends(get_db),
):
    """Get all users."""
    users = user_service.get_users(db, skip=skip, limit=limit)
    return users


@app.get("/users/{user_id}", response_model=User)
def read_user(user_id: int, db: Session = Depends(get_db)):
    """Get user by ID."""
    db_user = user_service.get_user(db, user_id=user_id)
    if db_user is None:
        raise HTTPException(status_code=404, detail="User not found")
    return db_user


@app.post("/users/", response_model=User)
def create_user(user: UserCreate, db: Session = Depends(get_db)):
    """Create a new user."""
    db_user = user_service.get_user_by_email(db, email=user.email)
    if db_user:
        raise HTTPException(status_code=400, detail="Email already registered")
    return user_service.create_user(db=db, user=user)


@app.delete("/users/{user_id}")
def delete_user(user_id: int, db: Session = Depends(get_db)):
    """Delete a user."""
    success = user_service.delete_user(db, user_id=user_id)
    if not success:
        raise HTTPException(status_code=404, detail="User not found")
    return {"status": "success", "message": "User deleted"}


# TODO: Add authentication endpoints
# TODO: Add more API endpoints for other resources
# FIXME: Implement proper error handling
# FIXME: Add request validation


def main():
    """Run the application."""
    logger.info("Starting application")
    uvicorn.run("src.main:app", host="0.0.0.0", port=8000, reload=True)


if __name__ == "__main__":
    main() 