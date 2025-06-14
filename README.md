# rust-wallpaper

A wallpaper service for **GNOME** that changes wallpaper according to **time of day, month, and weather**.

---

## ⚙️ Setup & Configuration

### 1. Configure weather service in `wallpaper_config.json`

A. Using Open-meteo.com:
- Set your latitude and longitude in the `wallpaper_config_json` (open_meteo_lat & open_meteo_long) as text values
B. Using Weatherapi.com:
- Create a free account at [https://api.weatherapi.com](https://api.weatherapi.com).
- Generate your **API key** and insert it as the value of `"api_key"` in the `wallpaper_config.json`. If you don't want to take weather into account, leave api_key (or location) value empty, this will omit the calls to weather API and leave the current_weather set to 'Clear' always
- Set your city name.

### 2. Move the configuration

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

### 3. Create the Systemd User Service

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

### 4. Build and Deploy

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

## Updating

- When updating, just repull the repository and run ./redeploy
