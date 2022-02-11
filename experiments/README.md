# Requirements

```bash
sudo apt install libgeos-dev
```

# Basic one-pass statistics
Simple single pass counter, decodes planet file without resolving node ID -> position. Counts the number of features and the number of tags each feature has.

```bash
# Use count1a or count1b for different PBF parsers
RUSTFLAGS='-Ctarget-cpu=native' cargo run --release count1a planet.osm.pbf
```

# Two-pass Way Nodes Resolution
First pass generates a cache file with `node IDs->(lat,lng)`. The second pass reiterates the planet file, resolving geolocation of each node using node cache, and computes metrics using one of the modes:
* `resolve` -- Resolve each node ID to lat/lng without any extra memory allocations
* `vector` -- Allocate a Rust vector of lat/lng pairs
* `geometry` -- Allocate a GEOS geometry from the Rust vector


```bash
RUSTFLAGS='-Ctarget-cpu=native' cargo run --release \
    count2 resolve planet.osm.pbf nodes.cache
```

# Node Usage by ways
Analyze which nodes (IDs) are used by ways.

First graph shows a histogram of way lengths - each bucket representing how many ways have that many nodes. We see that the vast majority of ways have 5 nodes (probably buildings), and most ways have less than 10-15 nodes each.

Second graph analyses the distribution of node IDs for each way by computing the min and max ID for each way, and subtracting `max-min`. The histogram is logarithm-based. Each bucket shows the number of ways with the similar node ID concentration, and the average number of nodes per way in that bucket. We see that the vast majority of ways' node IDs are highly localized - all within a 1,000. Only about 10-15% of ways span more than a million IDs, with ~15-20 nodes on average.

```bash
RUSTFLAGS='-Ctarget-cpu=native' cargo run --release node-dist planet.osm.pbf
```

<details>

```
Total ways: 833,487,337

Number of nodes in a way. Each '∎' represents 7,203,307 features.
     value           count                          distribution                   
--------------- ---------------  --------------------------------------------------
              0               0: 
              1              11: 
              2      59,140,254: ∎∎∎∎∎∎∎∎
              3      31,344,351: ∎∎∎∎
              4      23,307,883: ∎∎∎
              5     360,165,358: ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
              6      39,973,121: ∎∎∎∎∎
              7      65,339,896: ∎∎∎∎∎∎∎∎∎
              8      24,271,197: ∎∎∎
              9      45,029,746: ∎∎∎∎∎∎
             10      17,130,082: ∎∎
             11      23,316,285: ∎∎∎
             12      11,956,899: ∎
             13      16,999,114: ∎∎
             14       8,769,941: ∎
             15       9,889,431: ∎
             16       6,473,079: 
             17       9,220,092: ∎
             18       5,041,101: 
             19       5,263,776: 
             20       8,218,211: ∎
             21       4,343,472: 
             22       3,247,165: 
             23       3,187,884: 
             24       2,638,702: 
             25       2,656,125: 
             26       2,211,292: 
             27       2,144,617: 
             28       1,868,759: 
             29       1,873,605: 
             30       1,609,188: 
             31       1,560,057: 
             32       1,419,100: 
             33       1,388,588: 
             34       1,226,875: 
             35       1,183,029: 
             36       1,086,454: 
             37       1,112,230: 
             38         960,173: 
             39         925,488: 
             40         851,487: 
             41         846,254: 
             42         763,416: 
             43         739,905: 
             44         687,421: 
             45         674,023: 
             46         625,220: 
             47         603,939: 
             48         565,798: 
             49         552,427: 
            50+      19,084,816: ∎∎

Distance between min and max Node ID in a way feature, on a log scale. Each '∎' represents 4,291,025 features.
     value          count / avg size                          distribution                   
--------------- -------------------------  --------------------------------------------------
              1      20,594,609 /    2.0 : ∎∎∎∎
              1               0 /    NaN : 
              2               0 /    NaN : 
              2       9,387,104 /    2.7 : ∎∎
              3     214,551,297 /    4.9 : ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
              4      14,223,682 /    4.9 : ∎∎∎
              5      31,551,604 /    6.3 : ∎∎∎∎∎∎∎
              6      34,282,210 /    7.0 : ∎∎∎∎∎∎∎
              8      22,163,676 /    7.7 : ∎∎∎∎∎
             11      23,998,917 /    8.3 : ∎∎∎∎∎
             14      19,009,197 /    9.3 : ∎∎∎∎
             18      24,523,180 /   10.7 : ∎∎∎∎∎
             23      17,871,306 /    9.3 : ∎∎∎∎
             30      17,062,299 /    9.6 : ∎∎∎
             39      15,090,799 /    9.9 : ∎∎∎
             51      14,517,915 /   10.3 : ∎∎∎
             67      11,934,776 /   10.7 : ∎∎
             87      10,870,342 /   11.0 : ∎∎
            112      12,388,663 /   10.7 : ∎∎
            146      10,820,745 /   11.3 : ∎∎
            190      10,660,247 /   11.6 : ∎∎
            247       9,742,111 /   12.3 : ∎∎
            321       8,931,988 /   12.6 : ∎∎
            418       7,909,918 /   13.2 : ∎
            543       7,164,753 /   13.7 : ∎
            706       6,363,612 /   14.4 : ∎
            917       5,857,254 /   14.7 : ∎
          1,193       5,316,034 /   15.2 : ∎
          1,550       4,912,612 /   15.5 : ∎
          2,015       4,612,692 /   15.5 : ∎
          2,620       4,461,578 /   15.2 : ∎
          3,406       4,387,168 /   15.0 : ∎
          4,428       4,390,510 /   14.8 : ∎
          5,756       4,365,111 /   14.9 : ∎
          7,483       4,457,214 /   15.2 : ∎
          9,728       4,517,869 /   15.4 : ∎
         12,646       4,698,524 /   15.6 : ∎
         16,440       4,772,348 /   15.8 : ∎
         21,372       4,868,878 /   15.5 : ∎
         27,784       4,779,670 /   15.1 : ∎
         36,119       4,565,709 /   15.3 : ∎
         46,955       4,174,931 /   15.6 : 
         61,041       3,692,701 /   15.7 : 
         79,353       3,266,204 /   15.8 : 
        103,159       2,875,546 /   15.8 : 
        134,107       2,534,537 /   16.1 : 
        174,339       2,167,506 /   16.2 : 
        226,641       1,870,262 /   17.2 : 
        294,633       1,617,246 /   18.1 : 
        383,022       1,346,718 /   18.8 : 
        497,929       1,179,388 /   20.3 : 
        647,308       1,037,919 /   22.1 : 
        841,500         975,813 /   24.1 : 
      1,093,951         992,642 /   23.9 : 
      1,422,136       1,006,062 /   23.7 : 
      1,848,776       1,099,770 /   22.8 : 
      2,403,409       1,145,722 /   24.0 : 
      3,124,432       1,010,825 /   23.9 : 
      4,061,762         968,559 /   23.8 : 
      5,280,290         969,648 /   24.1 : 
      6,864,377         971,199 /   23.3 : 
      8,923,690         976,789 /   24.3 : 
     11,600,797         933,925 /   22.7 : 
     15,081,037         917,369 /   21.9 : 
     19,605,348         939,710 /   21.9 : 
     25,486,952         969,254 /   21.3 : 
     33,133,038         993,382 /   21.1 : 
     43,072,949       1,046,851 /   19.7 : 
     55,994,833       1,126,171 /   18.7 : 
     72,793,283       1,205,862 /   17.9 : 
     94,631,268       1,288,447 /   17.7 : 
    123,020,649       1,403,301 /   17.3 : 
    159,926,844       1,593,151 /   16.6 : 
    207,904,897       1,896,821 /   16.0 : 
    270,276,366       2,262,676 /   15.7 : 
    351,359,276       2,608,408 /   15.2 : 
    456,767,058       3,134,844 /   15.2 : 
    593,797,176       3,824,414 /   15.2 : 
    771,936,328       4,830,260 /   15.0 : ∎
  1,003,517,227       6,243,533 /   14.9 : ∎
  1,304,572,395       7,862,508 /   15.5 : ∎
  1,695,944,114       9,565,589 /   15.5 : ∎∎
  2,204,727,348      11,597,241 /   15.9 : ∎∎
  2,866,145,552      13,571,512 /   17.1 : ∎∎∎
  3,725,989,218      16,410,093 /   18.1 : ∎∎∎
  4,843,785,983      18,254,566 /   20.0 : ∎∎∎∎
  6,296,921,778      20,095,215 /   22.3 : ∎∎∎∎
  8,185,998,311      16,347,379 /   28.9 : ∎∎∎
 10,641,797,804         106,747 /   34.7 : 
Complete in 214.8 seconds
```
</details>
