---
title: 'Display a Notification'
description: 'Send a desktop notification from Windows or Ubuntu.'
---

## Windows

```yaml
shell:

button:
  - platform: shell
    name: 'Display Notification'
    command: |-
      $Program = "UbiHome"
      $ToastTitle = "Hello World!"
      $ToastText = "This is a test."

      [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null
      $Template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02)

      $RawXml = [xml] $Template.GetXml()
      ($RawXml.toast.visual.binding.text|where {$_.id -eq "1"}).AppendChild($RawXml.CreateTextNode($ToastTitle)) > $null
      ($RawXml.toast.visual.binding.text|where {$_.id -eq "2"}).AppendChild($RawXml.CreateTextNode($ToastText)) > $null

      $SerializedXml = New-Object Windows.Data.Xml.Dom.XmlDocument
      $SerializedXml.LoadXml($RawXml.OuterXml)

      $Toast = [Windows.UI.Notifications.ToastNotification]::new($SerializedXml)
      $Toast.Tag = $Program
      $Toast.Group = $Program
      $Toast.ExpirationTime = [DateTimeOffset]::Now.AddMinutes(1)

      $Notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($Program)
      $Notifier.Show($Toast);
```

> Sample script from: [https://den.dev/blog/powershell-windows-notification/](https://den.dev/blog/powershell-windows-notification/)

## Ubuntu

```yaml
shell:

# This is a pure sample (not tested yet!)
button:
  - platform: shell
    name: 'Display Notification'
    command: |-
      zenity --notification --text "Hello World from UbiHome!"
```

> Sample script from: [https://superuser.com/questions/31917/is-there-a-way-to-show-notification-from-bash-script-in-ubuntu](https://superuser.com/questions/31917/is-there-a-way-to-show-notification-from-bash-script-in-ubuntu))

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/features/entities/button/">Button</a>
  <a href="/features/platforms/shell/">Shell</a>
</div>
