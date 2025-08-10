from gcontributor.CustTypes import CommitDict
from datetime import datetime

class DataAccess:
    def __init__(self, dbLocation: str):
        self.dbLocation = dbLocation
        pass

    def createDict(self, commits: CommitDict)->None:
        pass

    def readDate(self, date: datetime)->int:
        pass

    def getStatus(self)->bool:
        pass

    def hasRun(self, date: datetime)->bool:
        pass

    def logRun(self, date: datetime)->None:
        pass