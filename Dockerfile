FROM python:3.9-buster
LABEL org.opencontainers.image.authors="team@eunomia.dev"

VOLUME [ "/knowledge-base/data" ]

EXPOSE 4100

# Copy things

COPY . /knowledge-base
WORKDIR /knowledge-base

# Install requirements

RUN pip install -r requirements.txt

# Specify entrypoint

ENTRYPOINT [ "python","entry.py" ]
