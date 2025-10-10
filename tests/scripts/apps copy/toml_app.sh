
# cargo clean 
# cargo build --release --features 'metrics para' 


echo "ark" 
declare -a b_ark=(580)
for i in {0..5}
do 
echo "$i"
for j in "${b_ark[@]}"
do
./target/release/coral -d ./tests/test_docs/toml/t3.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps_re/toml_t3_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/toml_t3_coral ./target/release/coral -d ./tests/test_docs/toml/t3.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps_re/toml_t3_coral.txt --prove


./target/release/coral -d ./tests/test_docs/toml/t3.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps_re/toml_t3_coral.txt --verify
done 
done 

echo "coral" 
declare -a b_coral=(554)
for i in {0..5}
do 
echo "$i"
for j in "${b_coral[@]}"
do
./target/release/coral -d ./tests/test_docs/toml/t2.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps_re/toml_t2_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/toml_t2_coral ./target/release/coral -d ./tests/test_docs/toml/t2.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps_re/toml_t2_coral.txt --prove

./target/release/coral -d ./tests/test_docs/toml/t2.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps_re/toml_t2_coral.txt --verify
done 
done 

echo "small" 
declare -a b_small=(267)
for i in {0..5}
do 
echo "$i"
for j in "${b_small[@]}"
do
./target/release/coral -d ./tests/test_docs/toml/t1.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps_re/toml_t1_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/toml_t1_coral ./target/release/coral -d ./tests/test_docs/toml/t1.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps_re/toml_t1_coral.txt --prove

./target/release/coral -d ./tests/test_docs/toml/t1.txt -g ./grammars/toml.pest -b "$j" -m ./tests/results/timings/apps_re/toml_t1_coral.txt --verify
done 
done 

