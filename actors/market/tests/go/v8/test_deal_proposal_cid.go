package main

import (
	"bytes"
	"encoding/hex"
	"flag"
	"fmt"

	market8 "github.com/filecoin-project/go-state-types/builtin/v8/market"
)

func main() {
	var data string

	flag.StringVar(&data, "data", "", "ipld encoded bytes in hex")

	flag.Parse()

	dataBytes, err := hex.DecodeString(data)
	if err != nil {
		panic(err)
	}

	buf := new(bytes.Buffer)
	buf.Write(dataBytes)

	var dealProposal market8.DealProposal
	if err := dealProposal.UnmarshalCBOR(buf); err != nil {
		panic(err)
	}

	cid, err := dealProposal.Cid()
	if err != nil {
		panic(err)
	}

	fmt.Println(cid)
}
