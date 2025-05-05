from asyncio import sleep
import fcntl
import os
import signal
from subprocess import PIPE, Popen
import time

DEFAULT_CONFIG = """
ubihome:
  name: new_awesome

logger:
  level: DEBUG

shell: 
  type: bash

button: 
 - platform: shell
   id: my_button
   name: "Write Hello World"
   command: "echo 'Hello World!' >> test.log"

sensor:
  - platform: shell
    name: "RAM Usage"
    id: ram_usage
    icon: mdi:memory
    state_class: "measurement"
    unit_of_measurement: "%"
    update_interval: 30s # 0 only executes it once and assumes a long running processes.
    command: |-
      cat testfile

gpio:
  device: raspberryPi

mqtt:
  broker: 127.0.0.1
  username: test
  password: test

power_utils:

api:
  encryption:
    key: "token"
"""

class UbiHome(object):
    
    def __init__(self, arguments, config = None):
        self.config = config if config else DEFAULT_CONFIG
        self.arguments = arguments
        self.configuration_file = f"config{os.getpid()}.yaml"


    def __enter__(self):
        with open(self.configuration_file, "w") as f:
            f.write(self.config)

        print(os.getcwd())
        self.process = Popen(
            f"./ubihome -c " + self.configuration_file + " " + self.arguments,
            shell=True,
            stdout=PIPE,
            stderr=PIPE,
            cwd=os.getcwd(),
            preexec_fn=os.setsid
        )
        return self
    
    async def __aenter__(self):
        self.__enter__()
        await sleep(0.5)
        return self

    async def __aexit__(self, exctype, value, tb):
        await sleep(1)
        self.__exit__(exctype, value, tb)

    def __exit__(self, exctype, value, tb):
        os.remove(self.configuration_file)
        stderr = self.get_stderr()
        print(self.get_stdout())
            
        self.process.terminate()
        # Make sure its really dead:
        try: 
          os.killpg(os.getpgid(self.process.pid), signal.SIGTERM)  # Send the signal to all the process groups
        except Exception:
          pass
        if stderr:
            raise Exception(f"Error: {stderr}")

    def get_stdout(self):
        return self.__non_block_read(self.process.stdout)

    def get_stderr(self):
        return self.__non_block_read(self.process.stderr)
    
    @staticmethod
    def __non_block_read(output):
        if output.closed:
            return ""
        fd = output.fileno()
        fl = fcntl.fcntl(fd, fcntl.F_GETFL)
        fcntl.fcntl(fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)
        try:
            return output.read().decode()
        except:
            return ""

def run_ubihome(arguments, config = None) -> str: 
    with UbiHome(arguments, config) as ubihome:
      stdout, stderr = ubihome.process.communicate()
      if stderr:
          raise Exception(f"Error: {stderr.decode().strip()}")

      return stdout.decode()


def wait_and_get_file(file_path, timeout=10):
    """
    Wait for a file to be created or modified.
    """
    start_time = time.time()
    while not os.path.exists(file_path):
        if time.time() - start_time > timeout:
            raise TimeoutError(f"File {file_path} was not created within {timeout} seconds.")
        time.sleep(0.1)
    return open(file_path, "r").read()