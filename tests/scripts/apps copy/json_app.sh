
# cargo clean 
# cargo build --release --features 'metrics para' 


# echo "citi" 
# declare -a b_citi=(611)
# for i in {0..2}
# do 
# echo "$i"
# for j in "${b_citi[@]}"
# do
# ./target/release/coral -d ./tests/test_docs/json/bank_citi.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_bank_citi_coral.txt --commit

# RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/json_bank_citi_coral ./target/release/coral -d ./tests/test_docs/json/bank_citi.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_bank_citi_coral.txt --prove

# ./target/release/coral -d ./tests/test_docs/json/bank_citi.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_bank_citi_coral.txt --verify

# done 
# done 

# echo "plaid" 
# declare -a b_plaid=(141)
# for i in {0..2}
# do 
# echo "$i"
# for j in "${b_plaid[@]}"
# do
# ./target/release/coral -d ./tests/test_docs/json/bank_plaid.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_bank_plaid_coral.txt --commit

# RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/json_bank_plaid_coral ./target/release/coral -d ./tests/test_docs/json/bank_plaid.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_bank_plaid_coral.txt --prove

# ./target/release/coral -d ./tests/test_docs/json/bank_plaid.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_bank_plaid_coral.txt --verify
# done 
# done 

# echo "veratad" 
# declare -a b_dmv=(305)
# for i in {0..2}
# do 
# echo "$i"
# for j in "${b_dmv[@]}"
# do
# ./target/release/coral -d ./tests/test_docs/json/dmv_veratad.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_dmv_veratad_coral.txt --commit

# RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/json_dmv_veratad_coral ./target/release/coral -d ./tests/test_docs/json/dmv_veratad.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_dmv_veratad_coral.txt --prove

# ./target/release/coral -d ./tests/test_docs/json/dmv_veratad.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_dmv_veratad_coral.txt --verify

# done 
# done 

echo "dk" 
declare -a b_dk=(641)
for i in {0..2}
do 
echo "$i"
for j in "${b_dk[@]}"
do
./target/release/coral -d ./tests/test_docs/json/draftgroups_dk.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_dk_coral.txt --commit

RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/json_dk_coral ./target/release/coral -d ./tests/test_docs/json/draftgroups_dk.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_dk_coral.txt --prove

./target/release/coral -d ./tests/test_docs/json/draftgroups_dk.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_dk_coral.txt --verify
done 
done 

# echo "hibps" 
# declare -a b_hs=(191)
# for i in {0..2}
# do 
# echo "$i"
# for j in "${b_hs[@]}"
# do
# ./target/release/coral -d ./tests/test_docs/json/hibp_small.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_hibp_small_coral.txt --commit

# RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/json_hibps_coral ./target/release/coral -d ./tests/test_docs/json/hibp_small.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_hibp_small_coral.txt --prove

# ./target/release/coral -d ./tests/test_docs/json/hibp_small.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_hibp_small_coral.txt --verify
# done 
# done 

# echo "jwt" 
# declare -a b_jwt=(201)
# for i in {0..2}
# do 
# echo "$i"
# for j in "${b_jwt[@]}"
# do
# ./target/release/coral -d ./tests/test_docs/json/jwt.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_jwt_coral.txt --commit

# RUST_BACKTRACE=1 gtime -v -a -o ./tests/results/memory/apps_re/json_jwt_coral ./target/release/coral -d ./tests/test_docs/json/jwt.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_jwt_coral.txt --prove

# ./target/release/coral -d ./tests/test_docs/json/jwt.txt -g ./grammars/json.pest -b "$j" -m ./tests/results/timings/apps_re/json_jwt_coral.txt --verify
# done 
# done 

