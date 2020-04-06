// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

module rxpath_test;
    reg clk = 0;
    always #5 clk = !clk;

    reg rx_wire = 1;
    wire out_bit;
    wire valid_now;
    wire byte_start;

    rxpath dut(
        .clk_8mhz(clk),
        .rx_wire(rx_wire),
        .out_bit(out_bit),
        .valid_now(valid_now),
        .byte_start(byte_start)
    );

    initial begin
        $dumpfile("rxpath.lxt");
        $dumpvars(0, rxpath_test);

        #7
        rx_wire <= 0;
        #80
        rx_wire <= 1;
        #80
        rx_wire <= 0;
        #80
        rx_wire <= 1;
        #80
        rx_wire <= 0;
        #80
        rx_wire <= 0;
        #80
        rx_wire <= 1;
        #80
        rx_wire <= 0;
        #80
        rx_wire <= 1;
        #80
        rx_wire <= 1;

        //#207
        #100
        rx_wire <= 0;
        #80
        rx_wire <= 1;
        #80
        rx_wire <= 1;
        #80
        rx_wire <= 0;
        #80
        rx_wire <= 0;
        #80
        rx_wire <= 0;
        #80
        rx_wire <= 0;
        #80
        rx_wire <= 1;
        #80
        rx_wire <= 1;
        #80
        rx_wire <= 1;

        #200;
        $finish;
    end

endmodule
