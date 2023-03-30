FROM python:3.9-buster
LABEL org.opencontainers.image.authors="team@eunomia.dev"

VOLUME [ "/knowledge-base/data" ]

EXPOSE 4100

# Copy things

COPY . /knowledge-base
WORKDIR /knowledge-base

# Install requirements

RUN pip install -r requirements.txt
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Build the web server
RUN cd web-server && $HOME/.cargo/bin/cargo install --profile release --path .
RUN cp $HOME/.cargo/bin/web-server /knowledge-base/web-server-executable
RUN cd web-server && $HOME/.cargo/bin/cargo clean
# Specify entrypoint

ENTRYPOINT [ "python","entry.py" ]
