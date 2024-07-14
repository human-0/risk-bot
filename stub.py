import base64
import random

from wasmtime import Store, Module, ValType, WasiConfig, Engine, Linker, FuncType

MAX_CHARACTERS_READ = 1000000
READ_CHUNK_SIZE = 1024

class WasmBot:
    def __init__(self):
        engine = Engine()
        module = Module(engine, base64.b64decode(WASM))
 
        linker = Linker(engine)
        linker.define_wasi()
 
        self._store = Store(engine)
        wasi = WasiConfig()
        wasi.inherit_stdout()
        wasi.inherit_stderr()
        wasi.inherit_env()
        self._store.set_wasi(wasi)
 
 
        self._write_file = open(f"./io/to_engine.pipe", "wb")
        self._read_file = open(f"./io/from_engine.pipe", "rb")
 
        linker.define_func("env", "read_pipe", FuncType([ValType.i32()], [ValType.i32()]), self._read_pipe)
        linker.define_func("env", "write_pipe", FuncType([ValType.i32(), ValType.i32()], []), self._write_pipe)
        instance = linker.instantiate(self._store, module)

        self._run = instance.exports(self._store)["run"]
        self._memory = instance.exports(self._store)["memory"]

    def run(self):
        self._run(self._store, random.randint(-(2 ** 31), (2 ** 31 - 1)))

    def _read_pipe(self, ptr: int) -> int:
        buffer = bytearray()
        while len(buffer) < 7 + 1 and (len(buffer) == 0 or buffer[-1] != ord(",")):
            buffer.extend(self._read_file.read(1))
 
        if buffer[-1] == ord(","):
            size = int(buffer[0:-1].decode())
        else:
            print(buffer)
            raise RuntimeError
        
        if size > MAX_CHARACTERS_READ:
            raise RuntimeError
        
        # Read message.
        buffer = bytearray()
        while len(buffer) < size:
            buffer.extend(bytearray(self._read_file.read(min((size - len(buffer)), READ_CHUNK_SIZE))))
 
        self._memory.write(self._store, buffer, ptr)
        return size

    def _write_pipe(self, ptr: int, size: int):
        buffer = self._memory.read(self._store, ptr, ptr + size)
 
        self._write_file.write(f"{size},".encode())
        self._write_file.write(buffer)
        self._write_file.flush()

if __name__ == "__main__":
    bot = WasmBot()
    bot.run()

