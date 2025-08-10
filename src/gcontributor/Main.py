import datetime
import schedule
from typing import *
from time import sleep

from gcontributor.Converter import IConverter 
from gcontributor.Commiter import Commiter
from gcontributor.DataAccess import DataAccess

class Main:
    def __init__(self, converter: IConverter, commiter: Commiter, data_accessor: DataAccess):
        """
        Class that organizes the lifecycle flow of the programming meant to be a long running program
        """
        self._converter = converter
        self._commiter = commiter
        self._data_accesor = data_accessor
        self._schedule_time = "06:00"

    def run(self)->None:
        """
        Sets up the schedule to run the _runCommit method. Tries to setup data storage if not already implemented
        """
        if not self._data_accesor.getStatus():
            self.setupPlan()
        if not self._data_accesor.hasRun():
            self._runCommit()

        schedule.every().data.at(self._schedule_time).do(self._runCommit, self) # figure out if I need this like in javascript or if it autobinds

    def _runCommit(self)->None:
        """
        Holds the logic to run the commit as scheduled by the scheduler
        """
        today = datetime.date.today()
        if self._data_accesor.hasRun(today):
            return

        commit_num = self._data_accesor.readDate(today)
        self._commiter.commit(commit_num)
        self._data_accesor.logRun(today)

    def setupPlan(self)->None:
        """
        Calls the setup plan stage and then ensures that it gets written through the DataAccess
        """
        commit_plan = self._converter.convert()
        self._data_accesor.createDict(commit_plan)


def main():
    #TODO going to need a parser to parse and construct Main correctly
    #TODO make sure this is singleton
    main = Main()
    main.run()

if __name__ == "__main__":
    main()