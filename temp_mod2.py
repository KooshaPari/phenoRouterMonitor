print("module name", __name__)
from dataclasses import dataclass
@dataclass
class C: x:int
print("class module", C.__module__)
