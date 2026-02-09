"""Pydantic schemas for API requests/responses."""

from datetime import datetime
from typing import Optional

from pydantic import BaseModel, Field


class TestModuleBase(BaseModel):
    name: str
    description: Optional[str] = None


class TestModuleCreate(TestModuleBase):
    pass


class TestModule(TestModuleBase):
    id: int
    created_at: datetime
    num_tests: int = 0
    passing_tests: int = 0

    class Config:
        from_attributes = True


class TestCaseBase(BaseModel):
    name: str
    description: Optional[str] = None
    test_type: Optional[str] = None


class TestCaseCreate(TestCaseBase):
    module_id: int


class TestCase(TestCaseBase):
    id: int
    module_id: int
    created_at: datetime
    last_run: Optional[datetime] = None
    last_result: Optional[bool] = None

    class Config:
        from_attributes = True


class TestRunBase(BaseModel):
    passed: bool
    execution_time_ms: Optional[float] = None
    error_message: Optional[str] = None
    git_commit: Optional[str] = None


class TestRunCreate(TestRunBase):
    test_case_id: int


class TestRun(TestRunBase):
    id: int
    test_case_id: int
    run_date: datetime

    class Config:
        from_attributes = True


class ExampleBase(BaseModel):
    name: str
    description: Optional[str] = None
    input_file_path: Optional[str] = None
    element_type: Optional[str] = None
    num_nodes: Optional[int] = None
    num_elements: Optional[int] = None
    num_dofs: Optional[int] = None


class ExampleCreate(ExampleBase):
    pass


class Example(ExampleBase):
    id: int
    created_at: datetime
    num_validations: int = 0
    last_validation: Optional[datetime] = None

    class Config:
        from_attributes = True


class ValidationResultBase(BaseModel):
    metric_name: str
    computed_value: float
    analytical_value: Optional[float] = None
    relative_error: Optional[float] = None
    passed: bool
    tolerance: Optional[float] = None
    git_commit: Optional[str] = None


class ValidationResultCreate(ValidationResultBase):
    example_id: int


class ValidationResult(ValidationResultBase):
    id: int
    example_id: int
    run_date: datetime

    class Config:
        from_attributes = True


class KPIBase(BaseModel):
    total_tests: int
    passing_tests: int
    test_coverage_percent: Optional[float] = None
    num_element_types: Optional[int] = None
    lines_of_code: Optional[int] = None
    avg_test_time_ms: Optional[float] = None
    git_commit: Optional[str] = None


class KPICreate(KPIBase):
    pass


class KPI(KPIBase):
    id: int
    date: datetime

    class Config:
        from_attributes = True


class DashboardStats(BaseModel):
    """Dashboard overview statistics."""

    total_tests: int
    passing_tests: int
    failing_tests: int
    pass_rate: float
    total_examples: int
    validated_examples: int
    total_modules: int
    avg_test_time_ms: float
    last_update: datetime
    supported_elements: list[str]
    lines_of_code: int


class ValidationSummary(BaseModel):
    """Summary of validation results for an example."""

    example_name: str
    element_type: str
    num_validations: int
    all_passed: bool
    max_error_percent: float
    avg_error_percent: float
    last_run: datetime
