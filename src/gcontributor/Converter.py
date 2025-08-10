from abc import abstractmethod
from gcontributor.CustTypes import CommitDict

class IConverter:
    @abstractmethod
    def convert(self, srcUri: str) -> CommitDict: 
        """
        Converts a file URI into a plan of commits into a dictionary of commits to acheive desired
        plan. Will return in the form of a dict with keys being the dates to integers being the number of commits.

        Need to consider abstracting the source uri out if we want that to be text passed into the program directly
        at some point.
        """
        raise NotImplementedError("Abstract method must be implemented")