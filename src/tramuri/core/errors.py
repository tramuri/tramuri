"""Typed errors and stable CLI exit codes."""


class TramuriError(Exception):
    """Base error shown without a traceback in the CLI."""

    exit_code = 1


class UsageError(TramuriError):
    exit_code = 2


class ProjectStateError(TramuriError):
    exit_code = 3


class ProviderConflictError(TramuriError):
    exit_code = 4


class ValidationError(TramuriError):
    exit_code = 5


class StorageError(TramuriError):
    exit_code = 6


class UnsafePathError(ProviderConflictError):
    pass
