# Basic one-pass statistics
Simple single pass counter, decodes planet file without resolving node ID -> position. Counts the number of features and the number of tags each feature has.

```bash
RUSTFLAGS='-Ctarget-cpu=native' cargo run --release count1 planet.osm.pbf
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
Total ways: 761835990

Number of nodes in a way. Each '∎' represents 6,549,184 features.
     value           count                          distribution                   
--------------- ---------------  --------------------------------------------------
              0               0: 
              1               3: 
              2      54,202,389: ∎∎∎∎∎∎∎∎
              3      28,786,864: ∎∎∎∎
              4      21,559,433: ∎∎∎
              5     327,459,209: ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
              6      37,075,505: ∎∎∎∎∎
              7      59,086,898: ∎∎∎∎∎∎∎∎∎
              8      22,555,414: ∎∎∎
              9      40,817,447: ∎∎∎∎∎∎
             10      15,989,415: ∎∎
             11      21,179,801: ∎∎∎
             12      11,111,148: ∎
             13      15,349,941: ∎∎
             14       8,152,855: ∎
             15       9,079,426: ∎
             16       6,011,938: 
             17       8,638,958: ∎
             18       4,679,004: 
             19       4,857,851: 
             20       7,343,145: ∎
             21       4,007,691: 
             22       3,002,888: 
             23       2,938,137: 
             24       2,438,035: 
             25       2,450,320: 
             26       2,043,928: 
             27       1,978,952: 
             28       1,725,071: 
             29       1,728,453: 
             30       1,485,775: 
             31       1,440,080: 
             32       1,313,287: 
             33       1,281,259: 
             34       1,133,624: 
             35       1,092,111: 
             36       1,001,760: 
             37       1,027,493: 
             38         886,065: 
             39         854,666: 
             40         787,025: 
             41         776,200: 
             42         705,539: 
             43         683,460: 
             44         634,945: 
             45         622,525: 
             46         577,530: 
             47         557,584: 
             48         522,767: 
             49         511,144: 
            50+      17,691,032: ∎∎

Distance between min and max Node ID in a way feature, on a log scale. Each '∎' represents 3,729,013 features.
     value          count / avg size                          distribution                   
--------------- -------------------------  --------------------------------------------------
              1      17,929,707 /    2.0 : ∎∎∎∎
              1               0 /    NaN : 
              2               0 /    NaN : 
              2       8,419,968 /    2.7 : ∎∎
              3     186,450,694 /    4.9 : ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
              4      12,854,311 /    4.9 : ∎∎∎
              5      27,423,079 /    6.2 : ∎∎∎∎∎∎∎
              6      30,483,624 /    6.9 : ∎∎∎∎∎∎∎∎
              8      20,171,532 /    7.5 : ∎∎∎∎∎
             11      22,222,078 /    8.0 : ∎∎∎∎∎
             14      17,976,405 /    9.1 : ∎∎∎∎
             18      23,182,026 /   10.3 : ∎∎∎∎∎∎
             23      17,333,541 /    9.0 : ∎∎∎∎
             30      16,670,832 /    9.3 : ∎∎∎∎
             39      14,814,080 /    9.6 : ∎∎∎
             51      14,307,668 /    9.9 : ∎∎∎
             67      11,800,526 /   10.3 : ∎∎∎
             87      10,771,187 /   10.7 : ∎∎
            112      12,341,776 /   10.4 : ∎∎∎
            146      10,785,202 /   11.1 : ∎∎
            190      10,631,290 /   11.5 : ∎∎
            247       9,716,121 /   12.1 : ∎∎
            321       8,896,423 /   12.5 : ∎∎
            418       7,860,537 /   13.2 : ∎∎
            543       7,109,220 /   13.7 : ∎
            706       6,294,576 /   14.5 : ∎
            917       5,761,114 /   14.9 : ∎
          1,193       5,174,928 /   15.5 : ∎
          1,550       4,737,753 /   15.8 : ∎
          2,015       4,389,910 /   15.9 : ∎
          2,620       4,176,354 /   15.8 : ∎
          3,406       4,057,464 /   15.5 : ∎
          4,428       4,006,266 /   15.4 : ∎
          5,756       3,962,789 /   15.4 : ∎
          7,483       4,063,817 /   15.6 : ∎
          9,728       4,153,967 /   15.7 : ∎
         12,646       4,320,524 /   15.8 : ∎
         16,440       4,367,607 /   16.0 : ∎
         21,372       4,429,112 /   15.7 : ∎
         27,784       4,353,168 /   15.3 : ∎
         36,119       4,163,767 /   15.5 : ∎
         46,955       3,838,914 /   15.7 : ∎
         61,041       3,420,215 /   16.0 : 
         79,353       3,057,281 /   16.1 : 
        103,159       2,718,237 /   16.1 : 
        134,107       2,431,670 /   16.3 : 
        174,339       2,089,413 /   16.3 : 
        226,641       1,827,028 /   17.3 : 
        294,633       1,569,239 /   18.2 : 
        383,022       1,276,759 /   19.2 : 
        497,929       1,113,767 /   20.8 : 
        647,308         979,201 /   22.6 : 
        841,500         948,705 /   24.2 : 
      1,093,951         967,024 /   24.2 : 
      1,422,136         977,930 /   24.0 : 
      1,848,776       1,046,501 /   22.8 : 
      2,403,409       1,058,884 /   23.9 : 
      3,124,432         922,537 /   23.7 : 
      4,061,762         909,583 /   23.8 : 
      5,280,290         900,182 /   24.1 : 
      6,864,377         896,895 /   23.2 : 
      8,923,690         903,948 /   24.3 : 
     11,600,797         859,340 /   22.5 : 
     15,081,037         848,045 /   21.8 : 
     19,605,348         863,285 /   21.8 : 
     25,486,952         902,536 /   21.3 : 
     33,133,038         925,375 /   21.0 : 
     43,072,949         975,374 /   19.7 : 
     55,994,833       1,053,306 /   18.7 : 
     72,793,283       1,133,560 /   17.9 : 
     94,631,268       1,216,358 /   17.9 : 
    123,020,649       1,333,480 /   17.6 : 
    159,926,844       1,501,721 /   16.9 : 
    207,904,897       1,804,822 /   16.0 : 
    270,276,366       2,153,238 /   15.6 : 
    351,359,276       2,503,650 /   15.1 : 
    456,767,058       2,997,652 /   15.3 : 
    593,797,176       3,688,697 /   15.3 : 
    771,936,328       4,653,662 /   15.2 : ∎
  1,003,517,227       6,025,687 /   15.1 : ∎
  1,304,572,395       7,653,155 /   15.7 : ∎∎
  1,695,944,114       9,322,958 /   15.9 : ∎∎
  2,204,727,348      11,295,094 /   16.3 : ∎∎∎
  2,866,145,552      13,080,837 /   17.6 : ∎∎∎
  3,725,989,218      15,512,650 /   18.7 : ∎∎∎∎
  4,843,785,983      17,154,848 /   21.0 : ∎∎∎∎
  6,296,921,778      17,485,315 /   24.1 : ∎∎∎∎
  8,185,998,311       8,472,489 /   31.5 : ∎∎
Complete in 201.2 seconds
```
</details>
