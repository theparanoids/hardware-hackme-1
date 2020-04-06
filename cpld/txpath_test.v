// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

module txpath_test;
    reg clk = 0;
    always #5 clk = !clk;

    wire tx_wire;
    reg which_byte = 0;
    reg trigger = 0;
    wire done;

    txpath dut(
        .clk_8mhz(clk),
        .which_byte(which_byte),
        .trigger(trigger),
        .tx_wire(tx_wire),
        .done(done)
    );

    initial begin
        $dumpfile("txpath.lxt");
        $dumpvars(0, txpath_test);

        #5
        trigger <= 1;
        #10
        trigger <= 0;

        #1000;
        which_byte <= 1;
        trigger <= 1;
        #10
        trigger <= 0;

        #1000;
        $finish;
    end

endmodule
