# RAM Monorepo

RAM monorepo implements components of Topology protocol stack.

## Running `ramd`

To build `ramd`, Rust (version 1.77.2 or later) is required. Please make sure Rust is installed and then run:

```
make run-ramd
```

It will run `ramd` and generate default config and necessary files. To view these files, use:

```
cd $HOME/.ramd
ls
```

(Optional) If you wish to change the location of these files, you can set `RAMD_DIR_NAME` environment variable in .env file as demonstrated [here](./.env.example). After configuring .env and running `ramd`, you can access the files with:

```
cd $HOME/{RAMD_DIR_NAME}
ls
```

If you encounter any issues, please feel free to [reach out](#contact) to us.

## Testing `ramd`

You can test `ramd` using shell scripts and example live objects located in [tests](./tests) directory. The original code of the examples can be found [here](https://github.com/jihoonsong/live-object-sdk).

To test `ramd`, open a terminal and run:

```
make run-ramd
```

In another terminal, you can send JSON-RPC requests via cURL.

### Sum Live Object

To create a Sum live object, run:

```
./tests/live-object-create.sh sum
```

To execute the Sum live object and sum two integers, run:

```
./tests/live-object-execute.sh sum 3 4 
```

You can replace `3` and `4` with any integers you'd like.

### GCounter Live Object

To create a GCounter live object, run:

```
./tests/live-object-create.sh gcounter
```

To execute the GCounter live object and add an integer, run:

```
./tests/live-object-execute.sh gcounter 1
```

You can replace `1` with any non-negative integer you'd like.

## Contributing

We are committed to community-driven development and welcome feedback and contributions from anyone on the internet!

If you're interested in collaborating with us, please refer to [CONTRIBUTING.md](./CONTRIBUTING.md) for more details.

## Contact

You can join our [Discord](https://discord.gg/hMsQas3Vw9) to ask questions or engage in discussions.

## License

RAM monorepo is licensed under the MIT License.
