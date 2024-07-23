FROM ubuntu:24.04@sha256:e3f92abc0967a6c19d0dfa2d55838833e947b9d74edbcb0113e48535ad4be12a

RUN apt-get update
RUN apt-get install -y --no-install-recommends ca-certificates clang curl libssl-dev pkg-config git dialog
RUN curl --proto '=https' --tlsv1.2 --retry 10 --retry-connrefused -fsSL 'https://sh.rustup.rs' | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN curl -L http://install.athenavm.org | bash && ~/.athena/bin/athup

WORKDIR /root/program

ENV CARGO_TERM_COLOR=always
ENTRYPOINT [ "/root/.athena/bin/cargo-athena" ]
CMD [ "athena" ]
