FROM python:3.11-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PORT=8080 \
    DATABASE_URL=sqlite:////tmp/validation_results.db

WORKDIR /app

RUN groupadd --system app && useradd --system --gid app app

COPY requirements.txt README.md ./
COPY main.py ./main.py
COPY database.py ./database.py
COPY schemas.py ./schemas.py
COPY nastran_converter.py ./nastran_converter.py
COPY templates ./templates
COPY static ./static
COPY scripts ./scripts
COPY validation_results.db ./validation_results.db
COPY entrypoint.sh ./entrypoint.sh

RUN pip install --upgrade pip \
    && pip install -r requirements.txt \
    && chmod +x /app/entrypoint.sh \
    && chown -R app:app /app

USER app

EXPOSE 8080

CMD ["/app/entrypoint.sh"]
