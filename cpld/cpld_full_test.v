// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

module BUFG(I, O);
    input I;
    output O;

    assign O = I;
endmodule

module cpld_full_test;
    reg clk = 0;
    always #5 clk = !clk;

    reg rx_wire = 1;
    wire tx_wire;

    top dut(
        .clk_(clk),
        .uart_rx(rx_wire),
        .uart_tx(tx_wire)
    );

    initial begin
        $dumpfile("cpld_full.lxt");
        $dumpvars(0, cpld_full_test);

        ///// CORRECT

        #7
        rx_wire <= 0;
        #80 // Bit 0
        rx_wire <= 0;
        #80 // Bit 1
        rx_wire <= 1;
        #80 // Bit 2
        rx_wire <= 0;
        #80 // Bit 3
        rx_wire <= 0;
        #80 // Bit 4
        rx_wire <= 0;
        #80 // Bit 5
        rx_wire <= 1;
        #80 // Bit 6
        rx_wire <= 0;
        #80 // Bit 7
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 8
        rx_wire <= 0;
        #80 // Bit 9
        rx_wire <= 0;
        #80 // Bit 10
        rx_wire <= 0;
        #80 // Bit 11
        rx_wire <= 1;
        #80 // Bit 12
        rx_wire <= 0;
        #80 // Bit 13
        rx_wire <= 1;
        #80 // Bit 14
        rx_wire <= 1;
        #80 // Bit 15
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 16
        rx_wire <= 1;
        #80 // Bit 17
        rx_wire <= 1;
        #80 // Bit 18
        rx_wire <= 1;
        #80 // Bit 19
        rx_wire <= 1;
        #80 // Bit 20
        rx_wire <= 1;
        #80 // Bit 21
        rx_wire <= 0;
        #80 // Bit 22
        rx_wire <= 0;
        #80 // Bit 23
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 24
        rx_wire <= 1;
        #80 // Bit 25
        rx_wire <= 0;
        #80 // Bit 26
        rx_wire <= 1;
        #80 // Bit 27
        rx_wire <= 1;
        #80 // Bit 28
        rx_wire <= 1;
        #80 // Bit 29
        rx_wire <= 0;
        #80 // Bit 30
        rx_wire <= 1;
        #80 // Bit 31
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 32
        rx_wire <= 1;
        #80 // Bit 33
        rx_wire <= 1;
        #80 // Bit 34
        rx_wire <= 0;
        #80 // Bit 35
        rx_wire <= 0;
        #80 // Bit 36
        rx_wire <= 1;
        #80 // Bit 37
        rx_wire <= 0;
        #80 // Bit 38
        rx_wire <= 1;
        #80 // Bit 39
        rx_wire <= 1;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 40
        rx_wire <= 1;
        #80 // Bit 41
        rx_wire <= 0;
        #80 // Bit 42
        rx_wire <= 0;
        #80 // Bit 43
        rx_wire <= 1;
        #80 // Bit 44
        rx_wire <= 1;
        #80 // Bit 45
        rx_wire <= 0;
        #80 // Bit 46
        rx_wire <= 0;
        #80 // Bit 47
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 48
        rx_wire <= 0;
        #80 // Bit 49
        rx_wire <= 0;
        #80 // Bit 50
        rx_wire <= 1;
        #80 // Bit 51
        rx_wire <= 1;
        #80 // Bit 52
        rx_wire <= 0;
        #80 // Bit 53
        rx_wire <= 1;
        #80 // Bit 54
        rx_wire <= 1;
        #80 // Bit 55
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 56
        rx_wire <= 1;
        #80 // Bit 57
        rx_wire <= 1;
        #80 // Bit 58
        rx_wire <= 1;
        #80 // Bit 59
        rx_wire <= 1;
        #80 // Bit 60
        rx_wire <= 1;
        #80 // Bit 61
        rx_wire <= 0;
        #80 // Bit 62
        rx_wire <= 0;
        #80 // Bit 63
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        ///// WRONG

        #1000
        rx_wire <= 0;
        #80 // Bit 0
        rx_wire <= 1;
        #80 // Bit 1
        rx_wire <= 1;
        #80 // Bit 2
        rx_wire <= 0;
        #80 // Bit 3
        rx_wire <= 0;
        #80 // Bit 4
        rx_wire <= 0;
        #80 // Bit 5
        rx_wire <= 1;
        #80 // Bit 6
        rx_wire <= 0;
        #80 // Bit 7
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 8
        rx_wire <= 0;
        #80 // Bit 9
        rx_wire <= 0;
        #80 // Bit 10
        rx_wire <= 0;
        #80 // Bit 11
        rx_wire <= 1;
        #80 // Bit 12
        rx_wire <= 0;
        #80 // Bit 13
        rx_wire <= 1;
        #80 // Bit 14
        rx_wire <= 1;
        #80 // Bit 15
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 16
        rx_wire <= 1;
        #80 // Bit 17
        rx_wire <= 1;
        #80 // Bit 18
        rx_wire <= 1;
        #80 // Bit 19
        rx_wire <= 1;
        #80 // Bit 20
        rx_wire <= 1;
        #80 // Bit 21
        rx_wire <= 0;
        #80 // Bit 22
        rx_wire <= 0;
        #80 // Bit 23
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 24
        rx_wire <= 1;
        #80 // Bit 25
        rx_wire <= 0;
        #80 // Bit 26
        rx_wire <= 1;
        #80 // Bit 27
        rx_wire <= 1;
        #80 // Bit 28
        rx_wire <= 1;
        #80 // Bit 29
        rx_wire <= 0;
        #80 // Bit 30
        rx_wire <= 1;
        #80 // Bit 31
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 32
        rx_wire <= 1;
        #80 // Bit 33
        rx_wire <= 1;
        #80 // Bit 34
        rx_wire <= 0;
        #80 // Bit 35
        rx_wire <= 0;
        #80 // Bit 36
        rx_wire <= 1;
        #80 // Bit 37
        rx_wire <= 0;
        #80 // Bit 38
        rx_wire <= 1;
        #80 // Bit 39
        rx_wire <= 1;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 40
        rx_wire <= 1;
        #80 // Bit 41
        rx_wire <= 0;
        #80 // Bit 42
        rx_wire <= 0;
        #80 // Bit 43
        rx_wire <= 1;
        #80 // Bit 44
        rx_wire <= 1;
        #80 // Bit 45
        rx_wire <= 0;
        #80 // Bit 46
        rx_wire <= 0;
        #80 // Bit 47
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 48
        rx_wire <= 0;
        #80 // Bit 49
        rx_wire <= 0;
        #80 // Bit 50
        rx_wire <= 1;
        #80 // Bit 51
        rx_wire <= 1;
        #80 // Bit 52
        rx_wire <= 0;
        #80 // Bit 53
        rx_wire <= 1;
        #80 // Bit 54
        rx_wire <= 1;
        #80 // Bit 55
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #100
        rx_wire <= 0;
        #80 // Bit 56
        rx_wire <= 1;
        #80 // Bit 57
        rx_wire <= 1;
        #80 // Bit 58
        rx_wire <= 1;
        #80 // Bit 59
        rx_wire <= 1;
        #80 // Bit 60
        rx_wire <= 1;
        #80 // Bit 61
        rx_wire <= 0;
        #80 // Bit 62
        rx_wire <= 0;
        #80 // Bit 63
        rx_wire <= 0;
        #80
        rx_wire <= 1;

        #1000;
        $finish;
    end
endmodule
