# cargo clean 
# cargo build --release --features 'metrics para' 


echo "c1" 
declare -a b_c1=(56)
for i in {0..10}
do 
echo "$i"
for j in "${b_c1[@]}"
do
./target/release/coral -d ./tests/test_docs/c/c1.txt -g ./grammars/c_simple.pest -b "$j" -m ./tests/results/timings/apps_re/c_c1_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/c_c1_coral ./target/release/coral -d ./tests/test_docs/c/c1.txt -g ./grammars/c_simple.pest -b "$j" -m ./tests/results/timings/apps_re/c_c1_coral.txt --prove

./target/release/coral -d ./tests/test_docs/c/c1.txt -g ./grammars/c_simple.pest -b "$j" -m ./tests/results/timings/apps_re/c_c1_coral.txt --verify
done 
done 

echo "c2" 
declare -a b_c2=(237)
for i in {0..10}
do 
echo "$i"
for j in "${b_c2[@]}"
do
./target/release/coral -d ./tests/test_docs/c/c2.txt -g ./grammars/c_simple.pest -b "$j" -m ./tests/results/timings/apps_re/c_c2_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/c_c2_coral ./target/release/coral -d ./tests/test_docs/c/c2.txt -g ./grammars/c_simple.pest -b "$j" -m ./tests/results/timings/apps_re/c_c2_coral.txt --prove

./target/release/coral -d ./tests/test_docs/c/c2.txt -g ./grammars/c_simple.pest -b "$j" -m ./tests/results/timings/apps_re/c_c2_coral.txt --verify

done 
done 

echo "c3" 
declare -a b_c3=(399)
for i in {0..10}
do 
echo "$i"
for j in "${b_c3[@]}"
do
./target/release/coral -d ./tests/test_docs/c/c3.txt -g ./grammars/c_simple.pest -b "$j" -m ./tests/results/timings/apps_re/c_c3_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/c_c3_coral ./target/release/coral -d ./tests/test_docs/c/c3.txt -g ./grammars/c_simple.pest -b "$j" -m ./tests/results/timings/apps_re/c_c3_coral.txt --prove

./target/release/coral -d ./tests/test_docs/c/c3.txt -g ./grammars/c_simple.pest -b "$j" -m ./tests/results/timings/apps_re/c_c3_coral.txt --verify
done 
done 

echo "llvm" 
declare -a b_llvm=(1460)
for i in {0..10}
do 
echo "$i"
for j in "${b_llvm[@]}"
do
./target/release/coral -d ./tests/test_docs/c/llvm_test_puzzle.txt -g ./grammars/c_simple.pest -b "$j" -m ./tests/results/timings/apps_re/c_llvm_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/c_llvm_coral ./target/release/coral -d ./tests/test_docs/c/llvm_test_puzzle.txt -g ./grammars/c_simple.pest -b "$j" -m ./tests/results/timings/apps_re/c_llvm_coral.txt --prove

./target/release/coral -d ./tests/test_docs/c/llvm_test_puzzle.txt -g ./grammars/c_simple.pest -b "$j" -m ./tests/results/timings/apps_re/c_llvm_coral.txt --verify
done 
done 