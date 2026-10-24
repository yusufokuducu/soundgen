# SoundGen - QA report

Generated: 2026-10-24 08:31

## Duration / peak / LUFS / loop seam

```
id                                dur    peak    lufs frames    seam
weapon_pistol_fire               0.45   -1.00   -20.9  21600       -
weapon_rifle_fire                0.77   -1.00   -24.7  36960       -
weapon_shotgun_fire              1.00   -1.00   -25.2  48000       -
weapon_smg_fire                  0.33   -1.00   -22.4  15841       -
weapon_suppressed_fire           0.47   -1.00   -22.4  22560       -
weapon_dry_fire                  0.21   -1.00   -26.2  10080       -
weapon_reload_mag                0.67   -1.00   -22.1  32160       -
weapon_casing_drop               0.75   -1.00   -25.9  36000       -
weapon_bolt_pull                 0.65   -1.00   -24.0  31201       -
weapon_shell_insert              0.60   -1.00   -22.3  28801       -
zombie_growl_low                 1.55   -1.00   -18.0  74400       -
zombie_growl_high                1.05   -1.00   -18.4  50400       -
zombie_scream                    1.75   -1.00   -15.4  84000       -
zombie_pain_hit                  0.50   -1.00   -17.9  24000       -
zombie_attack_lunge              1.15   -1.00   -18.6  55200       -
zombie_bite                      0.55   -1.00   -21.8  26400       -
zombie_death_rattle              2.45   -2.00   -17.8 117600       -
zombie_idle_breath               2.75   -5.00   -21.0 132000       -
zombie_horde_bed                 3.65   -4.00   -20.1 175200       -
gore_flesh_hit                   0.39   -1.00   -20.0  18720       -
gore_flesh_slash                 0.33   -1.00   -22.1  15841       -
gore_bone_break                  0.35   -1.00   -26.1  16801       -
gore_head_pop                    0.51   -1.00   -20.7  24480       -
gore_body_fall                   0.65   -1.00   -20.1  31201       -
gore_blood_splat                 0.60   -1.00   -22.7  28801       -
gore_dismember                   0.57   -1.00   -20.3  27360       -
gore_zombie_crunch               0.49   -1.00   -18.2  23520       -
foley_footstep_concrete          0.31   -1.00   -27.1  14880       -
foley_footstep_grass             0.29   -1.00   -22.6  13920       -
foley_footstep_mud               0.31   -1.00   -20.4  14880       -
foley_footstep_metal             0.37   -1.00   -23.7  17760       -
foley_body_land                  0.83   -1.00   -21.4  39840       -
foley_cloth_rustle               0.47   -1.00   -21.6  22560       -
foley_door_creak                 2.15   -1.00   -17.2 103200       -
foley_debris_glass               0.67   -1.00   -28.1  32160       -
loop_propeller_small             4.00   -6.00   -16.3 192000    0.15
loop_propeller_large             6.00   -6.00   -17.8 288000    0.01
loop_drone_motor                 4.00   -6.00   -19.6 192000    0.57
loop_helicopter_far              6.67   -9.00   -23.9 320000    0.09
loop_wind_loop                   8.00   -8.00   -28.8 384000    0.00
loop_campfire                    8.00   -8.00   -30.1 384000    0.00
crate_wood_creak                 1.40   -1.00   -16.9  67200       -
crate_open_pop                   0.60   -1.00   -22.9  28801       -
crate_shatter                    1.15   -1.00   -27.1  55200       -
crate_lock_click                 0.20   -1.00   -24.8   9600       -
crate_reward_reveal              1.55   -1.00   -17.2  74400       -
crate_metal_open                 0.95   -1.00   -19.0  45600       -
celebrate_confetti_popper        1.35   -1.00   -27.1  64800       -
celebrate_party_horn             0.85   -1.00   -21.3  40800       -
celebrate_confetti_rain          3.05   -1.00   -27.7 146400       -
celebrate_fanfare_sting          1.05   -1.00   -13.7  50400       -
celebrate_coin_burst             0.95   -1.00   -23.8  45600       -
celebrate_level_up               1.75   -1.00   -16.6  84000       -
ui_click_soft                    0.18   -5.00   -30.1   8640       -
ui_click_hard                    0.25   -3.50   -25.6  12000       -
ui_hover_tick                    0.12   -6.00   -29.7   5760       -
ui_coin_pickup                   0.55   -4.00   -20.0  26400       -
ui_purchase                      0.60   -4.00   -21.5  28801       -
ui_error_buzz                    0.40   -4.50   -22.3  19200       -
ui_level_up_short                0.55   -4.00   -19.1  26400       -
ui_menu_whoosh                   0.33   -5.00   -22.9  15841       -
amb_wind_ruined_city             6.05   -3.00   -19.9 290400       -
amb_distant_battle               6.55   -3.00   -19.2 314400       -
amb_zombie_moans_far             7.05   -3.00   -17.9 338400       -
amb_underground_hum              6.05   -3.00   -18.9 290400       -
amb_fire_crackle                 5.05   -3.00   -21.3 242401       -

66 presets, total render 1814 ms, avg 27.5 ms
```

## Envelope and band split

```
id                             atk_ms dec_ms  crest   low%   mid%    hi%  air%
weapon_pistol_fire                3.2  203.0   19.7    75    22     2     1
weapon_rifle_fire                 2.8  282.5   23.2    54    40     4     2
weapon_shotgun_fire               3.0  488.1   22.0    76    23     1     0
weapon_smg_fire                   3.2   37.6   20.7    63    30     4     3
weapon_suppressed_fire            4.5  190.6   21.4    68    28     4     0
weapon_dry_fire                   0.7   16.6   24.6     1    27    49    23
weapon_reload_mag               263.2  144.6   22.0    49    35    14     2
weapon_casing_drop                0.5  361.2   24.2     1    44    49     7
weapon_bolt_pull                120.4  255.5   22.4     2    54    40     4
weapon_shell_insert               5.4  305.0   20.6    57    39     3     0
zombie_growl_low                 43.1 1024.9   16.4    68    30     2     0
zombie_growl_high                32.0  739.9   16.6    55    40     5     1
zombie_scream                   149.0 1222.2   14.0    13    72    14     2
zombie_pain_hit                  23.0  303.3   17.2    55    41     3     0
zombie_attack_lunge              66.9  770.6   17.3    55    40     4     0
zombie_bite                       3.3  203.2   21.5    24    62    12     1
zombie_death_rattle              50.8 1847.4   15.3    75    23     1     0
zombie_idle_breath              909.5 1473.4   15.8    41    49     9     1
zombie_horde_bed                794.0 2523.9   15.6    73    26     2     0
gore_flesh_hit                    6.0  151.9   18.3    67    30     3     0
gore_flesh_slash                  3.3  131.2   20.4    38    34    21     7
gore_bone_break                   1.4   80.4   24.4     1    23    57    20
gore_head_pop                     3.7  246.6   20.1    83    15     2     0
gore_body_fall                    5.4  263.1   17.7    91     9     0     0
gore_blood_splat                  7.2  401.7   20.2    53    41     6     0
gore_dismember                   12.8  303.1   20.1     4    59    32     4
gore_zombie_crunch                9.2  294.7   17.4    65    32     2     0
foley_footstep_concrete           1.2   36.1   25.5     1    22    49    29
foley_footstep_grass             24.4   85.2   20.9    14    48    33     5
foley_footstep_mud                6.9   84.6   18.8    58    41     2     0
foley_footstep_metal              1.2   61.8   22.0    60    31     7     2
foley_body_land                   5.9  269.8   18.2    89    10     1     0
foley_cloth_rustle               28.9  222.8   20.6     2    46    45     7
foley_door_creak                 16.1 1535.0   15.7    65    32     2     0
foley_debris_glass                1.2  297.0   25.8     1    36    43    20
loop_propeller_small           3299.8  700.2    9.7    84    14     2     0
loop_propeller_large           1847.1 4152.9   11.1    91     8     1     0
loop_drone_motor               2794.8 1205.2   13.0    30    56    12     3
loop_helicopter_far               4.0 6662.6   14.3    77    21     1     0
loop_wind_loop                    4.0 7994.0   20.1    76    22     2     0
loop_campfire                     4.0 7993.0   21.4    82    18     0     0
crate_wood_creak                198.5  809.1   14.8    33    61     5     1
crate_open_pop                    2.4  269.6   20.1    52    46     2     0
crate_shatter                     1.8  459.9   24.8     7    57    28     7
crate_lock_click                  0.8   65.6   23.2     4    47    42     7
crate_reward_reveal             251.9  942.1   16.3     5    69    24     2
crate_metal_open                304.8  272.9   18.0    44    51     4     1
celebrate_confetti_popper         3.2  580.1   24.4     1    30    53    16
celebrate_party_horn             12.3  488.8   19.1    41    54     5     1
celebrate_confetti_rain         699.9 1125.0   26.2     1    40    51     8
celebrate_fanfare_sting         263.2  543.4   12.4    11    76    11     1
celebrate_coin_burst              0.6  367.9   21.7     2    63    32     2
celebrate_level_up              627.7  826.1   15.9    11    67    19     3
ui_click_soft                     1.0   16.4   24.4     9    66    23     2
ui_click_hard                     2.0   32.4   21.4    32    56    10     2
ui_hover_tick                     1.8   12.5   23.0     2    48    45     6
ui_coin_pickup                    1.5  250.1   16.7     3    70    24     3
ui_purchase                      91.9  177.3   15.8     5    71    23     1
ui_error_buzz                    16.8  217.4   17.1    61    36     2     0
ui_level_up_short               123.5  186.8   15.8     4    71    24     2
ui_menu_whoosh                   34.3  186.7   17.2    26    59    13     2
amb_wind_ruined_city            656.2 4132.8   16.4    76    22     2     0
amb_distant_battle              942.2 4612.1   15.7    91     9     0     0
amb_zombie_moans_far           1292.7 5254.6   14.4    75    23     2     0
amb_underground_hum            2393.6 3012.2   15.4    83    16     1     0
amb_fire_crackle                657.0 2928.5   18.0    83    16     1     0
```

## Automatic checks

- [x] no NaN / infinite samples
- [x] peak at or below 0 dBFS (limiter + normalisation)
- [x] loop seam below 0.05
- [x] unique ids (no duplicates)

