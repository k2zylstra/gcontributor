import pytest
from gcontributor.Main import Main

class MockCommiter:
    pass

class MockConverter:
    pass

class MockDataAccess:
    pass


def test_run():
    """
    Ensures that setupPlan is called and runCommit is called it has not been run
    """
    mockCommiter = MockCommiter()
    mockDataAccesor = MockDataAccess()
    mockConverter = MockConverter()

    m = Main(mockConverter, mockCommiter, mockDataAccesor)
    m.run()
    #TODO finish here