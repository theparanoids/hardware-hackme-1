// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

module txpath(
    input clk_8mhz,
    input which_byte,
    input trigger,
    output reg tx_wire,
    output done
);

    initial tx_wire = 1;

    // Used to divide to get 1 MHz
    reg [2:0] cyc_counter = 0;
    // Used to remember which bit we should be outputting
    reg [3:0] bit_counter = 0;

    always @(posedge clk_8mhz) begin
        if (trigger == 1)
            cyc_counter <= 0;
        else
            cyc_counter <= cyc_counter + 1;
    end

    always @(posedge clk_8mhz) begin
        if (trigger == 1)
            bit_counter <= 1;
        else if (cyc_counter == 7)
            if (bit_counter == 10)
                bit_counter <= 0;
            else if (bit_counter != 0)
                bit_counter <= bit_counter + 1;
    end
    
    always @(*) begin
        if (which_byte == 0) begin
            // 'N'
            case (bit_counter)
                1: tx_wire <= 0;    // Start
                2: tx_wire <= 0;
                3: tx_wire <= 1;
                4: tx_wire <= 1;
                5: tx_wire <= 1;
                6: tx_wire <= 0;
                7: tx_wire <= 0;
                8: tx_wire <= 1;
                9: tx_wire <= 0;
                10: tx_wire <= 1;   // Stop
                default: tx_wire <= 1;  // Idle
            endcase
        end else begin
            // 'Y'
            case (bit_counter)
                1: tx_wire <= 0;    // Start
                2: tx_wire <= 1;
                3: tx_wire <= 0;
                4: tx_wire <= 0;
                5: tx_wire <= 1;
                6: tx_wire <= 1;
                7: tx_wire <= 0;
                8: tx_wire <= 1;
                9: tx_wire <= 0;
                10: tx_wire <= 1;   // Stop
                default: tx_wire <= 1;  // Idle
            endcase
        end
    end

    assign done = bit_counter == 10 && cyc_counter == 7;

endmodule
