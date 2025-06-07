# rust-wallpaper

A wallpaper service for **GNOME** that changes wallpaper according to **time of day, month, and weather**.

---

## ⚙️ Setup & Configuration

### 1. Configure `wallpaper_config.json`

- Create a free account at [https://api.weatherapi.com](https://api.weatherapi.com).
- Generate your **API key** and insert it as the value of `"api_key"` in the `wallpaper_config.json`.
- Set your city name.
- Move the file:

  ```bash
  mkdir -p ~/.config/rust-wallpaper/
  mv wallpaper_config.json ~/.config/rust-wallpaper/
  ```

#### Notes

- Modify the JSON entries to your preference.
- Wallpaper paths must be **full system paths**.
- Supported weather conditions:  
  `Clear`, `Cloudy`, `Overcast`, `Rain`, `Thunderstorm`, `Fog`, `Snow`
- If an exact match is not found for the current hour, month, or weather, the service tries:
  - Similar weather
  - Other weather
  - Different hour
  - Next month

---

### 2. Create the Systemd User Service

Create the file:

```bash
~/.config/systemd/user/wallpaperd.service
```

Paste the following content:

```ini
[Unit]
Description=Wallpaper Daemon
After=graphical-session.target

[Service]
ExecStart=/usr/local/bin/wallpaperd
Restart=on-failure

[Install]
WantedBy=default.target
```

---

### 3. Build and Deploy

From the repository root, run:

```bash
./redeploy.sh
```

> **Note:** You need **Rust** and **Cargo** installed.

---

## 🖼️ Behavior

- Checks every **5 minutes**.
- Starts automatically with the GNOME session.
- Changes wallpaper based on time and weather.

---

## 📝 Notes

- If you change the JSON config while the service is running, it will reload the config on the next run.
- Currently only works with **GNOME**.
- This project was created as a home project to learn **Rust**. It's my first Rust project, so it's not perfect, but it gets the job done.
