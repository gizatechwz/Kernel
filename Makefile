# kernelkite — developer tasks
#
# The Rust workspace builds on any OS. The `/proc` backend only does real work
# on Linux; elsewhere use fixture replay (the `samples` target works anywhere).

CARGO ?= cargo
