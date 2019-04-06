FROM scratch

ADD i2cbus /
ADD api.yaml /static/
ENV GFA_SERVER_PORT=8080
EXPOSE 8080

ENTRYPOINT ["/i2cbus"]
