// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

package main

import (
    "bytes"
    "fmt"
    "log"
    "os"
    "github.com/jacobsa/go-serial/serial"
    "github.com/pkg/term/termios"
)

type CTFSerial struct {
    Ser *os.File
}

func OpenCTFSerial(device_node_name string) *CTFSerial {
    ret := new(CTFSerial)

    options := serial.OpenOptions{
        PortName: device_node_name,
        BaudRate: 115200,
        DataBits: 8,
        StopBits: 1,
        InterCharacterTimeout: 1000,
    }

    port, err := serial.Open(options)
    if err != nil {
        log.Fatalf("serial.Open: %v", err)
    }

    // This is an awful hack that probably doesn't work on Windows.
    // We need this because we need the fd for tcdrain
    ret.Ser = port.(*os.File)

    return ret
}

func (ser *CTFSerial) Write(bytes []byte) {
    for _, b := range bytes {
        ser.Ser.Write([]byte{b})
        termios.Tcdrain(ser.Ser.Fd())
    }
}

func (ser *CTFSerial) Read(n int) []byte {
    buf := make([]byte, n)
    actuallen := 0
    for actuallen < n {
        thisreadlen, _ := ser.Ser.Read(buf[actuallen:])

        if thisreadlen == 0 {
            break
        }

        actuallen += thisreadlen
    }
    return buf[0:actuallen]
}

func (ser *CTFSerial) WriteAndCheck(b []byte) bool {
    bytes_with_all_crlf := crlf_hack(b)

    ser.Write(b)
    read := ser.Read(len(bytes_with_all_crlf))
    return bytes.Equal(read, bytes_with_all_crlf)
}

func crlf_hack(inp []byte) []byte {
    var outp []byte
    seencr := false

    for _, b := range inp {
        if b != '\r' && b != '\n' {
            if seencr {
                outp = append(outp, '\r')
                outp = append(outp, '\n')
            }
            seencr = false
            outp = append(outp, b)
        } else if b == '\r' {
            if seencr {
                outp = append(outp, '\r')
                outp = append(outp, '\n')
            }
            seencr = true
        } else if b == '\n' {
            outp = append(outp, '\r')
            outp = append(outp, '\n')
            seencr = false
        }
    }

    if seencr {
        outp = append(outp, '\r')
        outp = append(outp, '\n')
    }

    return outp
}

func main() {
    if len(os.Args) < 2 {
        fmt.Printf("Usage: %s serport\n", os.Args[0])
        return
    }

    ser := OpenCTFSerial(os.Args[1])
    ok := ser.WriteAndCheck([]byte("help\n"))
    fmt.Println(ok)
    fmt.Println(string(ser.Read(1000)))
    ok = ser.WriteAndCheck([]byte("version\n"))
    fmt.Println(ok)
    fmt.Println(string(ser.Read(1000)))
}
