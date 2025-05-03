API_CONFIG = """
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