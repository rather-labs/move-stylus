
<a name="hello_world_erc20"></a>

# Module `hello_world::erc20`



-  [Struct `TOTAL_SUPPLY`](#hello_world_erc20_TOTAL_SUPPLY)
-  [Struct `CONTRACT_INFO`](#hello_world_erc20_CONTRACT_INFO)
-  [Struct `ALLOWANCE_`](#hello_world_erc20_ALLOWANCE_)
-  [Struct `BALANCE_`](#hello_world_erc20_BALANCE_)
-  [Struct `TotalSupply`](#hello_world_erc20_TotalSupply)
-  [Struct `Info`](#hello_world_erc20_Info)
-  [Struct `Transfer`](#hello_world_erc20_Transfer)
-  [Struct `Approval`](#hello_world_erc20_Approval)
-  [Struct `Balance`](#hello_world_erc20_Balance)
-  [Struct `Allowance`](#hello_world_erc20_Allowance)
-  [Constants](#@Constants_0)
-  [Function `init`](#hello_world_erc20_init)
-  [Function `mint`](#hello_world_erc20_mint)
-  [Function `burn`](#hello_world_erc20_burn)
-  [Function `total_supply`](#hello_world_erc20_total_supply)
-  [Function `decimals`](#hello_world_erc20_decimals)
-  [Function `name`](#hello_world_erc20_name)
-  [Function `symbol`](#hello_world_erc20_symbol)
-  [Function `balance_of`](#hello_world_erc20_balance_of)
-  [Function `transfer`](#hello_world_erc20_transfer)
-  [Function `approve`](#hello_world_erc20_approve)
-  [Function `allowance`](#hello_world_erc20_allowance)
-  [Function `transfer_from`](#hello_world_erc20_transfer_from)


<pre><code><b>use</b> <a href="../dependencies/std/ascii.md#std_ascii">std::ascii</a>;
<b>use</b> <a href="../dependencies/std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../dependencies/std/vector.md#std_vector">std::vector</a>;
<b>use</b> <a href="../dependencies/stylus/dynamic_field.md#stylus_dynamic_field">stylus::dynamic_field</a>;
<b>use</b> <a href="../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id">stylus::dynamic_field_named_id</a>;
<b>use</b> <a href="../dependencies/stylus/event.md#stylus_event">stylus::event</a>;
<b>use</b> <a href="../dependencies/stylus/object.md#stylus_object">stylus::object</a>;
<b>use</b> <a href="../dependencies/stylus/table.md#stylus_table">stylus::table</a>;
<b>use</b> <a href="../dependencies/stylus/transfer.md#stylus_transfer">stylus::transfer</a>;
<b>use</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context">stylus::tx_context</a>;
</code></pre>



<a name="hello_world_erc20_TOTAL_SUPPLY"></a>

## Struct `TOTAL_SUPPLY`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/erc20.md#hello_world_erc20_TOTAL_SUPPLY">TOTAL_SUPPLY</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
</dl>


</details>

<a name="hello_world_erc20_CONTRACT_INFO"></a>

## Struct `CONTRACT_INFO`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/erc20.md#hello_world_erc20_CONTRACT_INFO">CONTRACT_INFO</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
</dl>


</details>

<a name="hello_world_erc20_ALLOWANCE_"></a>

## Struct `ALLOWANCE_`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/erc20.md#hello_world_erc20_ALLOWANCE_">ALLOWANCE_</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
</dl>


</details>

<a name="hello_world_erc20_BALANCE_"></a>

## Struct `BALANCE_`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/erc20.md#hello_world_erc20_BALANCE_">BALANCE_</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
</dl>


</details>

<a name="hello_world_erc20_TotalSupply"></a>

## Struct `TotalSupply`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/erc20.md#hello_world_erc20_TotalSupply">TotalSupply</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;<a href="../hello_world/erc20.md#hello_world_erc20_TOTAL_SUPPLY">hello_world::erc20::TOTAL_SUPPLY</a>&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>total: u256</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_erc20_Info"></a>

## Struct `Info`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/erc20.md#hello_world_erc20_Info">Info</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;<a href="../hello_world/erc20.md#hello_world_erc20_CONTRACT_INFO">hello_world::erc20::CONTRACT_INFO</a>&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code><a href="../hello_world/erc20.md#hello_world_erc20_name">name</a>: <a href="../dependencies/std/ascii.md#std_ascii_String">std::ascii::String</a></code>
</dt>
<dd>
</dd>
<dt>
<code><a href="../hello_world/erc20.md#hello_world_erc20_symbol">symbol</a>: <a href="../dependencies/std/ascii.md#std_ascii_String">std::ascii::String</a></code>
</dt>
<dd>
</dd>
<dt>
<code><a href="../hello_world/erc20.md#hello_world_erc20_decimals">decimals</a>: u8</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_erc20_Transfer"></a>

## Struct `Transfer`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/erc20.md#hello_world_erc20_Transfer">Transfer</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>from: <b>address</b></code>
</dt>
<dd>
</dd>
<dt>
<code>to: <b>address</b></code>
</dt>
<dd>
</dd>
<dt>
<code>value: u256</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_erc20_Approval"></a>

## Struct `Approval`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/erc20.md#hello_world_erc20_Approval">Approval</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>owner: <b>address</b></code>
</dt>
<dd>
</dd>
<dt>
<code>spender: <b>address</b></code>
</dt>
<dd>
</dd>
<dt>
<code>value: u256</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_erc20_Balance"></a>

## Struct `Balance`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/erc20.md#hello_world_erc20_Balance">Balance</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;<a href="../hello_world/erc20.md#hello_world_erc20_BALANCE_">hello_world::erc20::BALANCE_</a>&gt;</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_erc20_Allowance"></a>

## Struct `Allowance`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/erc20.md#hello_world_erc20_Allowance">Allowance</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;<a href="../hello_world/erc20.md#hello_world_erc20_ALLOWANCE_">hello_world::erc20::ALLOWANCE_</a>&gt;</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="hello_world_erc20_EInssuficientFunds"></a>



<pre><code><b>const</b> <a href="../hello_world/erc20.md#hello_world_erc20_EInssuficientFunds">EInssuficientFunds</a>: u64 = 1;
</code></pre>



<a name="hello_world_erc20_ENotAllowed"></a>



<pre><code><b>const</b> <a href="../hello_world/erc20.md#hello_world_erc20_ENotAllowed">ENotAllowed</a>: u64 = 2;
</code></pre>



<a name="hello_world_erc20_init"></a>

## Function `init`



<pre><code><b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_init">init</a>(_ctx: &<b>mut</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_init">init</a>(_ctx: &<b>mut</b> TxContext) {
    transfer::freeze_object(<a href="../hello_world/erc20.md#hello_world_erc20_Info">Info</a> {
        id: object::new_named_id&lt;<a href="../hello_world/erc20.md#hello_world_erc20_CONTRACT_INFO">CONTRACT_INFO</a>&gt;(),
        <a href="../hello_world/erc20.md#hello_world_erc20_name">name</a>: ascii::string(b"Test Coin"),
        <a href="../hello_world/erc20.md#hello_world_erc20_symbol">symbol</a>: ascii::string(b"TST"),
        <a href="../hello_world/erc20.md#hello_world_erc20_decimals">decimals</a>: 18,
    });
    transfer::share_object(<a href="../hello_world/erc20.md#hello_world_erc20_TotalSupply">TotalSupply</a> {
        id: object::new_named_id&lt;<a href="../hello_world/erc20.md#hello_world_erc20_TOTAL_SUPPLY">TOTAL_SUPPLY</a>&gt;(),
        total: 0,
    });
    transfer::share_object(<a href="../hello_world/erc20.md#hello_world_erc20_Allowance">Allowance</a> {
        id: object::new_named_id&lt;<a href="../hello_world/erc20.md#hello_world_erc20_ALLOWANCE_">ALLOWANCE_</a>&gt;(),
    });
    transfer::share_object(<a href="../hello_world/erc20.md#hello_world_erc20_Balance">Balance</a> {
        id: object::new_named_id&lt;<a href="../hello_world/erc20.md#hello_world_erc20_BALANCE_">BALANCE_</a>&gt;(),
    });
}
</code></pre>



</details>

<a name="hello_world_erc20_mint"></a>

## Function `mint`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_mint">mint</a>(to: <b>address</b>, amount: u256, <a href="../hello_world/erc20.md#hello_world_erc20_total_supply">total_supply</a>: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_TotalSupply">hello_world::erc20::TotalSupply</a>, balance: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_Balance">hello_world::erc20::Balance</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_mint">mint</a>(
    to: <b>address</b>,
    amount: u256,
    <a href="../hello_world/erc20.md#hello_world_erc20_total_supply">total_supply</a>: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_TotalSupply">TotalSupply</a>,
    balance: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_Balance">Balance</a>
) {
    <b>if</b> (field::exists_(&balance.id, to)) {
        <b>let</b> balance_amount = field::borrow_mut(&<b>mut</b> balance.id, to);
        *balance_amount = *balance_amount + amount;
    } <b>else</b> {
        field::add(&<b>mut</b> balance.id, to, amount);
    };
    <a href="../hello_world/erc20.md#hello_world_erc20_total_supply">total_supply</a>.total = <a href="../hello_world/erc20.md#hello_world_erc20_total_supply">total_supply</a>.total + amount;
    emit(<a href="../hello_world/erc20.md#hello_world_erc20_Transfer">Transfer</a> {
        from: @0x0,
        to,
        value: amount
    });
}
</code></pre>



</details>

<a name="hello_world_erc20_burn"></a>

## Function `burn`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_burn">burn</a>(from: <b>address</b>, amount: u256, <a href="../hello_world/erc20.md#hello_world_erc20_total_supply">total_supply</a>: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_TotalSupply">hello_world::erc20::TotalSupply</a>, balance: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_Balance">hello_world::erc20::Balance</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_burn">burn</a>(
    from: <b>address</b>,
    amount: u256,
    <a href="../hello_world/erc20.md#hello_world_erc20_total_supply">total_supply</a>: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_TotalSupply">TotalSupply</a>,
    balance: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_Balance">Balance</a>
) {
    <b>if</b> (amount &gt; 0 && !field::exists_(&balance.id, from)) {
        <b>abort</b>(<a href="../hello_world/erc20.md#hello_world_erc20_EInssuficientFunds">EInssuficientFunds</a>)
    };
    <b>let</b> balance_amount = field::borrow_mut(&<b>mut</b> balance.id, from);
    <b>if</b> (*balance_amount &lt; amount) {
        <b>abort</b>(<a href="../hello_world/erc20.md#hello_world_erc20_EInssuficientFunds">EInssuficientFunds</a>)
    };
    <b>if</b> (amount &gt; <a href="../hello_world/erc20.md#hello_world_erc20_total_supply">total_supply</a>.total) {
        *balance_amount = 0;
        <a href="../hello_world/erc20.md#hello_world_erc20_total_supply">total_supply</a>.total = 0;
    } <b>else</b> {
        *balance_amount = *balance_amount - amount;
        <a href="../hello_world/erc20.md#hello_world_erc20_total_supply">total_supply</a>.total = <a href="../hello_world/erc20.md#hello_world_erc20_total_supply">total_supply</a>.total - amount;
    };
    emit(<a href="../hello_world/erc20.md#hello_world_erc20_Transfer">Transfer</a> {
        from,
        to: @0x0,
        value: amount
    });
}
</code></pre>



</details>

<a name="hello_world_erc20_total_supply"></a>

## Function `total_supply`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_total_supply">total_supply</a>(t_supply: &<a href="../hello_world/erc20.md#hello_world_erc20_TotalSupply">hello_world::erc20::TotalSupply</a>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_total_supply">total_supply</a>(t_supply: &<a href="../hello_world/erc20.md#hello_world_erc20_TotalSupply">TotalSupply</a>): u256 {
    t_supply.total
}
</code></pre>



</details>

<a name="hello_world_erc20_decimals"></a>

## Function `decimals`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_decimals">decimals</a>(contract_info: &<a href="../hello_world/erc20.md#hello_world_erc20_Info">hello_world::erc20::Info</a>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_decimals">decimals</a>(contract_info: &<a href="../hello_world/erc20.md#hello_world_erc20_Info">Info</a>): u8 {
    contract_info.<a href="../hello_world/erc20.md#hello_world_erc20_decimals">decimals</a>
}
</code></pre>



</details>

<a name="hello_world_erc20_name"></a>

## Function `name`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_name">name</a>(contract_info: &<a href="../hello_world/erc20.md#hello_world_erc20_Info">hello_world::erc20::Info</a>): <a href="../dependencies/std/ascii.md#std_ascii_String">std::ascii::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_name">name</a>(contract_info: &<a href="../hello_world/erc20.md#hello_world_erc20_Info">Info</a>): String {
    contract_info.<a href="../hello_world/erc20.md#hello_world_erc20_name">name</a>
}
</code></pre>



</details>

<a name="hello_world_erc20_symbol"></a>

## Function `symbol`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_symbol">symbol</a>(contract_info: &<a href="../hello_world/erc20.md#hello_world_erc20_Info">hello_world::erc20::Info</a>): <a href="../dependencies/std/ascii.md#std_ascii_String">std::ascii::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_symbol">symbol</a>(contract_info: &<a href="../hello_world/erc20.md#hello_world_erc20_Info">Info</a>): String {
    contract_info.<a href="../hello_world/erc20.md#hello_world_erc20_symbol">symbol</a>
}
</code></pre>



</details>

<a name="hello_world_erc20_balance_of"></a>

## Function `balance_of`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_balance_of">balance_of</a>(account: <b>address</b>, balance: &<a href="../hello_world/erc20.md#hello_world_erc20_Balance">hello_world::erc20::Balance</a>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_balance_of">balance_of</a>(account: <b>address</b>, balance: &<a href="../hello_world/erc20.md#hello_world_erc20_Balance">Balance</a>): u256 {
    <b>if</b> (field::exists_(&balance.id, account)) {
        *field::borrow&lt;<a href="../hello_world/erc20.md#hello_world_erc20_BALANCE_">BALANCE_</a>, <b>address</b>, u256&gt;(&balance.id, account)
    } <b>else</b> {
        0
    }
}
</code></pre>



</details>

<a name="hello_world_erc20_transfer"></a>

## Function `transfer`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_transfer">transfer</a>(recipient: <b>address</b>, amount: u256, tx_context: &<a href="../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>, balance: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_Balance">hello_world::erc20::Balance</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_transfer">transfer</a>(
    recipient: <b>address</b>,
    amount: u256,
    tx_context: &TxContext,
    balance: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_Balance">Balance</a>,
): bool {
    <b>let</b> sender_balance = field::borrow_mut&lt;<a href="../hello_world/erc20.md#hello_world_erc20_BALANCE_">BALANCE_</a>, <b>address</b>, u256&gt;(
        &<b>mut</b> balance.id,
        tx_context.sender()
    );
    <b>if</b> (*sender_balance &lt; amount) {
        <b>abort</b>(<a href="../hello_world/erc20.md#hello_world_erc20_EInssuficientFunds">EInssuficientFunds</a>)
    };
    *sender_balance = *sender_balance - amount;
    <b>if</b> (field::exists_(&balance.id, recipient)) {
        <b>let</b> recipient_balance = field::borrow_mut(&<b>mut</b> balance.id, recipient);
        *recipient_balance = *recipient_balance + amount;
    } <b>else</b> {
        field::add(&<b>mut</b> balance.id, recipient, amount);
    };
    emit(<a href="../hello_world/erc20.md#hello_world_erc20_Transfer">Transfer</a> {
        from: tx_context.sender(),
        to: recipient,
        value: amount
    });
    <b>true</b>
}
</code></pre>



</details>

<a name="hello_world_erc20_approve"></a>

## Function `approve`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_approve">approve</a>(spender: <b>address</b>, amount: u256, <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_Allowance">hello_world::erc20::Allowance</a>, ctx: &<b>mut</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_approve">approve</a>(
    spender: <b>address</b>,
    amount: u256,
    <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_Allowance">Allowance</a>,
    ctx: &<b>mut</b> TxContext,
): bool {
    <b>let</b> spender_allowance = <b>if</b> (field::exists_(&<a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>.id, ctx.sender())) {
        field::borrow_mut&lt;<a href="../hello_world/erc20.md#hello_world_erc20_ALLOWANCE_">ALLOWANCE_</a>, <b>address</b>, Table&lt;<b>address</b>, u256&gt;&gt;(&<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>.id, ctx.sender())
    } <b>else</b> {
        field::add(
            &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>.id,
            ctx.sender(),
            table::new&lt;<b>address</b>, u256&gt;(ctx)
        );
        field::borrow_mut&lt;<a href="../hello_world/erc20.md#hello_world_erc20_ALLOWANCE_">ALLOWANCE_</a>, <b>address</b>, Table&lt;<b>address</b>, u256&gt;&gt;(&<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>.id, ctx.sender())
    };
    <b>if</b> (spender_allowance.contains(spender)) {
        <b>let</b> <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a> = spender_allowance.borrow_mut(spender);
        *<a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a> = amount;
    } <b>else</b> {
        spender_allowance.add(spender, amount);
    };
    emit(<a href="../hello_world/erc20.md#hello_world_erc20_Approval">Approval</a> {
        owner: ctx.sender(),
        spender,
        value: amount
    });
    <b>true</b>
}
</code></pre>



</details>

<a name="hello_world_erc20_allowance"></a>

## Function `allowance`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>(owner: <b>address</b>, spender: <b>address</b>, <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>: &<a href="../hello_world/erc20.md#hello_world_erc20_Allowance">hello_world::erc20::Allowance</a>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>(
    owner: <b>address</b>,
    spender: <b>address</b>,
    <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>: &<a href="../hello_world/erc20.md#hello_world_erc20_Allowance">Allowance</a>,
): u256 {
    <b>if</b> (field::exists_(&<a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>.id, owner)) {
        <b>let</b> owner_allowance = field::borrow&lt;<a href="../hello_world/erc20.md#hello_world_erc20_ALLOWANCE_">ALLOWANCE_</a>, <b>address</b>, Table&lt;<b>address</b>, u256&gt;&gt;(
            &<a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>.id,
            owner
        );
        *owner_allowance.borrow(spender)
    } <b>else</b> {
        0
    }
}
</code></pre>



</details>

<a name="hello_world_erc20_transfer_from"></a>

## Function `transfer_from`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_transfer_from">transfer_from</a>(sender: <b>address</b>, recipient: <b>address</b>, amount: u256, <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_Allowance">hello_world::erc20::Allowance</a>, balance: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_Balance">hello_world::erc20::Balance</a>, ctx: &<b>mut</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/erc20.md#hello_world_erc20_transfer_from">transfer_from</a>(
    sender: <b>address</b>,
    recipient: <b>address</b>,
    amount: u256,
    <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_Allowance">Allowance</a>,
    balance: &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_Balance">Balance</a>,
    ctx: &<b>mut</b> TxContext,
): bool {
    <b>if</b> (field::exists_(&<a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>.id, sender)) {
        <b>let</b> spender_allowance = field::borrow_mut&lt;<a href="../hello_world/erc20.md#hello_world_erc20_ALLOWANCE_">ALLOWANCE_</a>, <b>address</b>, Table&lt;<b>address</b>, u256&gt;&gt;(
            &<b>mut</b> <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a>.id,
            sender,
        );
        <b>let</b> <a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a> = spender_allowance.borrow_mut(ctx.sender());
        <b>if</b> (*<a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a> &lt; amount) {
            <b>abort</b>(<a href="../hello_world/erc20.md#hello_world_erc20_ENotAllowed">ENotAllowed</a>)
        };
        *<a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a> = *<a href="../hello_world/erc20.md#hello_world_erc20_allowance">allowance</a> - amount;
        <b>let</b> sender_balance = field::borrow_mut&lt;<a href="../hello_world/erc20.md#hello_world_erc20_BALANCE_">BALANCE_</a>, <b>address</b>, u256&gt;(
            &<b>mut</b> balance.id,
            sender
        );
        <b>if</b> (*sender_balance &lt; amount) {
            <b>abort</b>(<a href="../hello_world/erc20.md#hello_world_erc20_EInssuficientFunds">EInssuficientFunds</a>)
        };
        *sender_balance = *sender_balance - amount;
        <b>if</b> (field::exists_(&balance.id, recipient)) {
            <b>let</b> recipient_balance = field::borrow_mut(&<b>mut</b> balance.id, recipient);
            *recipient_balance = *recipient_balance + amount;
        } <b>else</b> {
            field::add(&<b>mut</b> balance.id, recipient, amount);
        };
    } <b>else</b> {
        <b>abort</b>(<a href="../hello_world/erc20.md#hello_world_erc20_ENotAllowed">ENotAllowed</a>)
    };
    emit(<a href="../hello_world/erc20.md#hello_world_erc20_Transfer">Transfer</a> {
        from: sender,
        to: recipient,
        value: amount
    });
    <b>true</b>
}
</code></pre>



</details>
