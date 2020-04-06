// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

import com.fazecast.jSerialComm.*;
import java.io.ByteArrayOutputStream;

public class CTFSerial {
    public SerialPort ser;

    public CTFSerial(String device_node_name) {
        this.ser = SerialPort.getCommPort(device_node_name);
        this.ser.setBaudRate(115200);
        this.ser.openPort();
        this.ser.setComPortTimeouts(SerialPort.TIMEOUT_READ_BLOCKING | SerialPort.TIMEOUT_WRITE_BLOCKING, 1000, 0);
    }

    public void write(byte[] bytes) {
        for (int i = 0; i < bytes.length; i++) {
            this.ser.writeBytes(bytes, 1, i);
        }
    }

    public void write(String str) {
        try {
            this.write(str.getBytes("UTF-8"));
        } catch (java.io.UnsupportedEncodingException e) {
            // WTF?
        }
    }

    public byte[] read(int n) {
        byte[] tmp = new byte[n];
        int actuallen = this.ser.readBytes(tmp, n);
        byte[] ret = new byte[actuallen];
        System.arraycopy(tmp, 0, ret, 0, actuallen);
        return ret;
    }

    public boolean write_and_check(byte[] bytes) {
        byte[] bytes_with_all_crlf = CTFSerial.crlf_hack(bytes);

        this.write(bytes);
        byte[] read = this.read(bytes_with_all_crlf.length);
        return java.util.Arrays.equals(read, bytes_with_all_crlf);
    }

    public boolean write_and_check(String str) {
        try {
            return this.write_and_check(str.getBytes("UTF-8"));
        } catch (java.io.UnsupportedEncodingException e) {
            // WTF?
            return false;
        }
    }

    private static byte[] crlf_hack(byte[] inp) {
        ByteArrayOutputStream outp = new ByteArrayOutputStream();
        boolean seencr = false;

        for (byte b : inp) {
            if (b != '\r' && b != '\n') {
                if (seencr) {
                    outp.write('\r');
                    outp.write('\n');
                }
                seencr = false;
                outp.write(b);
            } else if (b == '\r') {
                if (seencr) {
                    outp.write('\r');
                    outp.write('\n');
                }
                seencr = true;
            } else if (b == '\n') {
                outp.write('\r');
                outp.write('\n');
                seencr = false;
            }
        }

        if (seencr) {
            outp.write('\r');
            outp.write('\n');
        }

        return outp.toByteArray();
    }

    public static void main(String[] args) {
        if (args.length < 1) {
            System.out.println("Usage: java CTFSerial serport");
            return;
        }

        CTFSerial ser = new CTFSerial(args[0]);
        boolean ok = ser.write_and_check("help\n");
        System.out.println(ok);
        byte[] read = ser.read(1000);
        System.out.println(new String(read, java.nio.charset.StandardCharsets.UTF_8));
        ok = ser.write_and_check("version\n");
        System.out.println(ok);
        read = ser.read(1000);
        System.out.println(new String(read, java.nio.charset.StandardCharsets.UTF_8));
    }
}
