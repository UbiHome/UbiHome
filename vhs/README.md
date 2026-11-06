# Demo GIF

Generates `documentation/public/assets/demo.gif` with [VHS](https://github.com/charmbracelet/vhs) from `demo.tape`. UbiHome is built inside the image, so only Docker is required.

```bash
cd vhs
docker compose run --rm --build generate

# Interactive
docker compose run --rm --build record
```
