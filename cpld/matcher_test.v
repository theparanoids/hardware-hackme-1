// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

module matcher_test;
    reg clk = 0;
    always #5 clk = !clk;

    // RX path
    reg rx_bit = 0;
    reg rx_bit_valid_now = 0;
    reg rx_byte_start = 0;

    // TX path
    wire tx_which_byte;
    wire tx_trigger;
    reg tx_done = 0;

    always @(posedge clk)
        tx_done <= tx_trigger;

    matcher dut(
        .clk(clk),

        .rx_bit(rx_bit),
        .rx_bit_valid_now(rx_bit_valid_now),
        .rx_byte_start(rx_byte_start),

        .tx_which_byte(tx_which_byte),
        .tx_trigger(tx_trigger),
        .tx_done(tx_done)
    );

    initial begin
        $dumpfile("matcher.lxt");
        $dumpvars(0, matcher_test);

        ///// CORRECT

        #5  // Bit 0
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 1
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 2
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 3
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 4
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 5
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 6
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 7
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 8
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 9
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 10
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 11
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 12
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 13
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 14
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 15
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 16
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 17
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 18
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 19
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 20
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 21
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 22
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 23
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 24
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 25
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 26
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 27
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 28
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 29
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 30
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 31
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 32
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 33
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 34
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 35
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 36
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 37
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 38
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 39
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 40
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 41
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 42
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 43
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 44
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 45
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 46
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 47
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 48
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 49
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 50
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 51
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 52
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 53
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 54
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 55
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 56
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 57
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 58
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 59
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 60
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 61
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 62
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 63
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        ///// WRONG

        #100; // Bit 0
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 1
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 2
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 3
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 4
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 5
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 6
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 7
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 8
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 9
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 10
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 11
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 12
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 13
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 14
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 15
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 16
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 17
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 18
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 19
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 20
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 21
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 22
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 23
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 24
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 25
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 26
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 27
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 28
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 29
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 30
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 31
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 32
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 33
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 34
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 35
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 36
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 37
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 38
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 39
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 40
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 41
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 42
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 43
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 44
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 45
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 46
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 47
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 48
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 49
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 50
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 51
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 52
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 53
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 54
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 55
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #10 // Bit 56
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 1;
        #10
        rx_bit_valid_now <= 0;
        rx_byte_start <= 0;

        #10 // Bit 57
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        rx_byte_start <= 0;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 58
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 59
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 60
        rx_bit <= 1;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 61
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 62
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;

        #10 // Bit 63
        rx_bit <= 0;
        rx_bit_valid_now <= 1;
        #10
        rx_bit_valid_now <= 0;


        #100;
        $finish;
    end
endmodule
