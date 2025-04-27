# Power Utilities

```yaml
power_utils:

button:
 - platform: power_utils
   name: "Power Off"
   action: poweroff

 - platform: power_utils
   name: "Reboot"
   action: reboot

 - platform: power_utils
   name: "Logout"
   action: logout

 - platform: power_utils
   name: "Hibernate"
   action: hibernate

 - platform: power_utils
   name: "Sleep"
   action: sleep
```

ESPHome reference: 
- https://esphome.io/components/button/restart.html
- https://esphome.io/components/button/shutdown