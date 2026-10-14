#!/usr/bin/env python3
import sys
import os

# We will implement the venv bootstrapping here if needed,
# or we can rely on an external wrapper. For now, this is the entry point.

from ui.app import ZaneMegApp

def main():
    print("Starting ZaneMegApp...")
    app = ZaneMegApp()
    app.mainloop()

if __name__ == "__main__":
    main()
