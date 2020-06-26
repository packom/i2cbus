FROM piersfinlayson/build-amd64:0.3.6 as i2cbus
WORKDIR /home/build/builds
RUN git clone https://github.com/packom/i2cbus
WORKDIR /home/build/builds/i2cbus
ARG TYPE
ARG TARGET
RUN cargo build $TYPE --target=$TARGET

FROM scratch
WORKDIR /
ARG TARGET
ARG COPY_TYPE
COPY --from=i2cbus /home/build/builds/i2cbus/target/$TARGET/$COPY_TYPE/i2cbus ./i2cbus
ARG DIR
ADD $DIR/api.yaml /static/
VOLUME ["/ssl"]
EXPOSE 8080
CMD ["/i2cbus"]
