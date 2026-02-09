"""Database models and session management."""

from datetime import datetime
from typing import Optional

from sqlalchemy import (
    Boolean,
    Column,
    DateTime,
    Float,
    ForeignKey,
    Integer,
    String,
    Text,
    create_engine,
)
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import relationship, sessionmaker

DATABASE_URL = "sqlite:///./validation_results.db"

engine = create_engine(DATABASE_URL, connect_args={"check_same_thread": False})
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
Base = declarative_base()


class TestModule(Base):
    """Test module/category."""

    __tablename__ = "test_modules"

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, unique=True, nullable=False, index=True)
    description = Column(Text)
    created_at = Column(DateTime, default=datetime.utcnow)

    # Relationships
    test_cases = relationship("TestCase", back_populates="module", cascade="all, delete-orphan")


class TestCase(Base):
    """Individual test case."""

    __tablename__ = "test_cases"

    id = Column(Integer, primary_key=True, index=True)
    module_id = Column(Integer, ForeignKey("test_modules.id"), nullable=False)
    name = Column(String, nullable=False, index=True)
    description = Column(Text)
    test_type = Column(String)  # unit, integration, end-to-end
    created_at = Column(DateTime, default=datetime.utcnow)

    # Relationships
    module = relationship("TestModule", back_populates="test_cases")
    runs = relationship("TestRun", back_populates="test_case", cascade="all, delete-orphan")


class TestRun(Base):
    """Individual test execution result."""

    __tablename__ = "test_runs"

    id = Column(Integer, primary_key=True, index=True)
    test_case_id = Column(Integer, ForeignKey("test_cases.id"), nullable=False)
    run_date = Column(DateTime, default=datetime.utcnow, index=True)
    passed = Column(Boolean, nullable=False)
    execution_time_ms = Column(Float)
    error_message = Column(Text)
    git_commit = Column(String)

    # Relationships
    test_case = relationship("TestCase", back_populates="runs")


class Example(Base):
    """Example problem (e.g., simple_truss.inp)."""

    __tablename__ = "examples"

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, unique=True, nullable=False, index=True)
    description = Column(Text)
    input_file_path = Column(String)
    element_type = Column(String)  # T3D2, B31, C3D8, etc.
    num_nodes = Column(Integer)
    num_elements = Column(Integer)
    num_dofs = Column(Integer)
    created_at = Column(DateTime, default=datetime.utcnow)

    # Relationships
    validations = relationship(
        "ValidationResult", back_populates="example", cascade="all, delete-orphan"
    )


class ValidationResult(Base):
    """Validation result comparing computed vs analytical."""

    __tablename__ = "validation_results"

    id = Column(Integer, primary_key=True, index=True)
    example_id = Column(Integer, ForeignKey("examples.id"), nullable=False)
    run_date = Column(DateTime, default=datetime.utcnow, index=True)
    metric_name = Column(String, nullable=False)  # displacement, stress, etc.
    computed_value = Column(Float, nullable=False)
    analytical_value = Column(Float)
    relative_error = Column(Float)
    passed = Column(Boolean, nullable=False)
    tolerance = Column(Float)
    git_commit = Column(String)

    # Relationships
    example = relationship("Example", back_populates="validations")


class KPI(Base):
    """Key Performance Indicator tracking over time."""

    __tablename__ = "kpis"

    id = Column(Integer, primary_key=True, index=True)
    date = Column(DateTime, default=datetime.utcnow, index=True)
    total_tests = Column(Integer, nullable=False)
    passing_tests = Column(Integer, nullable=False)
    test_coverage_percent = Column(Float)
    num_element_types = Column(Integer)
    lines_of_code = Column(Integer)
    avg_test_time_ms = Column(Float)
    git_commit = Column(String)


def init_db():
    """Initialize database tables."""
    Base.metadata.create_all(bind=engine)


def get_db():
    """Get database session."""
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
