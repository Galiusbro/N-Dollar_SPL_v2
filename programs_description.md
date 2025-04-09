# Описание программ в директории `programs/`

Этот документ содержит описание файлов для каждой программы Solana в директории `programs/`.

## 1. `programs/token-distributor/`

Программа для распределения токенов по разным адресам.

- **`Cargo.toml`**: Определяет метаданные пакета (имя, версия, описание), зависимости (`anchor-lang`, `anchor-spl`, `bonding-curve`) и конфигурацию сборки для программы `token-distributor`, написанной с использованием фреймворка Anchor. Указывает, что программа `bonding-curve` является локальной зависимостью. Включает CPI.
- **`Xargo.toml`**: Файл конфигурации для кросс-компиляции Rust-кода под архитектуру BPF, используемую в Solana.
- **`src/lib.rs`**: Основной файл исходного кода программы `token-distributor`.
  - Содержит логику инструкции `distribute_tokens`, которая распределяет токены с Associated Token Account (ATA), принадлежащего PDA этой программы (сиды: `b"distributor"`, `mint_key`).
  - Распределение: 20% пользователю (`user_token_account`), 30% на ATA программы `bonding-curve` (`bonding_curve_token_account`), 50% (остаток) на ATA казны реферальной программы (`referral_treasury_token_account`).
  - Использует PDA для подписи транзакций перевода токенов (CPI к SPL Token).
  - Создает необходимые ATA (`user_token_account`, `bonding_curve_token_account`, `referral_treasury_token_account`) с помощью `init_if_needed`, если они еще не существуют. Плательщиком за создание ATA выступает `user_authority`.
  - Определяет ID самой программы и ID реферальной программы (для `seeds::program` при проверке PDA казны реферальной программы).
  - Определяет структуру аккаунтов `DistributeTokens` и коды ошибок `ErrorCode`.

## 2. `programs/genesis/`

Программа для создания новых пользовательских SPL-токенов и их первоначальной настройки.

- **`Cargo.toml`**: Определяет метаданные и зависимости для программы `genesis`. Включает зависимости от `anchor-lang`, `anchor-spl`, `mpl-token-metadata` (для создания метаданных токена), а также локальные программы `n-dollar`, `liquidity-pool`, `token-distributor` и `bonding-curve`. Включает CPI для локальных зависимостей.
- **`Xargo.toml`**: Файл конфигурации для кросс-компиляции Rust-кода под архитектуру BPF Solana.
- **`src/lib.rs`**: Основной исходный код программы `genesis`.
  - Определяет инструкцию `create_user_token`, которая отвечает за создание нового SPL-токена.
  - Принимает от пользователя метаданные (имя, символ, URI), общее количество токенов (`total_supply`) и количество `N-Dollar` токенов для оплаты ренты (`n_dollar_amount`).
  - **Оплата ренты**:
    - Проверяет, что предоставленное количество `N-Dollar` (`n_dollar_amount`) и количество SOL в пуле ликвидности (`pool_sol_account`) достаточны для покрытия оценочной стоимости ренты за создание новых аккаунтов (Mint, Metadata, TokenInfo, ATA дистрибьютора).
    - Вызывает инструкцию `swap_ndollar_to_sol` программы `liquidity-pool` через CPI, чтобы обменять `n_dollar_amount` пользователя на SOL. Эти SOL поступают на системный аккаунт пользователя (`authority`).
  - **Создание токена**:
    - Инициализирует новый минт SPL-токена (`mint`) с 9 децималами. Авторитет минта и заморозки - пользователь (`authority`).
    - Создает аккаунт метаданных токена (`metadata`) с помощью программы `mpl_token_metadata`.
    - Инициализирует кастомный аккаунт `TokenInfo` (PDA, сиды: `b"token_info"`, `mint_key`) для хранения `mint`, `authority` и `total_supply`.
  - **Минтинг**:
    - Чеканит (mints) `total_supply` новых токенов на ATA (`distributor_token_account`), принадлежащий PDA программы `token-distributor` (сиды: `b"distributor"`, `mint_key`). Авторитетом для этой операции минта является пользователь (`authority`). ATA дистрибьютора создается (`init_if_needed`), если не существует.
  - **Обработка ренты**:
    - Рассчитывает SOL, фактически потраченные на ренту (`sol_used_for_rent`), сравнивая баланс `authority` до и после создания аккаунтов.
    - Возвращает излишек SOL (за вычетом минимально необходимой ренты для созданных аккаунтов + буфер) с аккаунта `authority` обратно в SOL-хранилище пула ликвидности (`pool_sol_account`).
  - Генерирует событие `TokenCreated`.
  - **Не вызывает** `token_distributor::distribute_tokens` (код закомментирован).
  - Определяет константы (оценки ренты, параметры токена), структуру аккаунта `TokenInfo`, коды ошибок `ErrorCode` и структуру аккаунтов `CreateUserToken`.

## 3. `programs/referral-program/`

Программа для обработки реферальных вознаграждений.

- **`Cargo.toml`**: Определяет метаданные и зависимости для реферальной программы. Зависит от `anchor-lang`, `anchor-spl` и локальной программы `token-distributor`. Включает CPI для `token-distributor`.
- **`Xargo.toml`**: Файл конфигурации для кросс-компиляции Rust-кода под архитектуру BPF Solana.
- **`src/lib.rs`**: Основной исходный код реферальной программы.
  - Определяет инструкцию `process_referral`.
  - Принимает аккаунты реферера (`referrer`) и двух рефералов (`referee1_token_account`, `referee2_token_account`).
  - **Казна**: Использует ATA (`referral_treasury_token_account`), принадлежащий PDA этой программы (сиды: `b"referral_treasury"`, `mint_key`). Этот ATA предназначен для хранения токенов, используемых для вознаграждений.
  - **Логика**:
    - Проверяет, достаточно ли средств в казне для выплаты вознаграждений обоим рефералам.
    - Переводит фиксированное вознаграждение (`REWARD_AMOUNT`, 1 токен) из казны на токен-аккаунты каждого из двух рефералов.
    - Использует PDA казны для подписи CPI вызовов к программе SPL Token для перевода средств.
  - **Не создает** реферальные аккаунты или ATA, предполагается, что они уже существуют.
  - Определяет структуру аккаунтов `ProcessReferral` и коды ошибок `ErrorCode`.

## 4. `programs/bonding-curve/`

Программа, реализующая линейную кривую связывания для обмена токена на N-Dollar.

- **`Cargo.toml`**: Определяет метаданные и зависимости для программы `bonding-curve`. Зависит от `anchor-lang`, `anchor-spl` и локальной программы `liquidity-pool`. Включает CPI для `liquidity-pool`.
- **`Xargo.toml`**: Файл конфигурации для кросс-компиляции Rust-кода под архитектуру BPF Solana.
- **`src/lib.rs`**: Основной исходный код программы `bonding-curve`.
  - **Концепция**: Реализует механизм ценообразования для нового токена (`TokenX`) против `N-Dollar`. Цена `TokenX` линейно растет с `0.00005 N$` до `1 N$` по мере продажи `30,000,000 TokenX` с кривой.
  - **Состояние (`BondingCurve` PDA)**: Хранит параметры кривой (начальный запас `TokenX`, наклон, начальная цена), адреса минтов, ATA, авторитет и bump. Сиды PDA: `b"bonding_curve"`, `mint.key()`.
  - **Инструкция `initialize_curve`**:
    - Инициализирует `BondingCurve` PDA.
    - Проверяет, что начальный баланс `TokenX` в `bonding_curve_token_account` (принадлежащем PDA) соответствует `30,000,000`.
    - Создает ATA (`n_dollar_treasury`) для хранения `N-Dollar`, полученных от продаж, принадлежащий PDA кривой.
  - **Инструкция `buy` (TokenX за N-Dollar)**:
    - Пользователь отправляет `N-Dollar` в `n_dollar_treasury`.
    - Программа рассчитывает количество `TokenX` к выдаче, интегрируя линейную функцию цены \(P(y) = my + c\), где \(y\) - количество уже проданных `TokenX`. Использует округление вверх (`ceil_div`) для расчета стоимости в `N-Dollar`.
    - Переводит рассчитанное количество `TokenX` из `bonding_curve_token_account` пользователю, подписывая PDA.
  - **Инструкция `sell` (TokenX за N-Dollar)**:
    - Пользователь отправляет `TokenX` на `bonding_curve_token_account`.
    - Программа рассчитывает количество `N-Dollar` к выдаче, интегрируя ту же функцию цены. Использует округление вниз (`floor_div`) для расчета выручки в `N-Dollar`.
    - Переводит рассчитанное количество `N-Dollar` из `n_dollar_treasury` пользователю, подписывая PDA.
  - Использует вычисления с фиксированной точкой (`u128`, `PRECISION_FACTOR`) для точности.
  - Определяет структуры аккаунтов `InitializeCurve`, `BuySell`, стейт `BondingCurve` и ошибки `BondingCurveError`.

## 5. `programs/n-dollar/`

Программа для создания токена N-Dollar и инициализации связанного с ним пула ликвидности.

- **`Cargo.toml`**: Определяет метаданные и зависимости для программы `n-dollar`. Зависит от `anchor-lang`, `anchor-spl`, `mpl-token-metadata` и локальной программы `liquidity-pool`. Включает CPI для `liquidity-pool`.
- **`Xargo.toml`**: Файл конфигурации для кросс-компиляции Rust-кода под архитектуру BPF Solana.
- **`src/lib.rs`**: Основной исходный код программы `n-dollar`.
  - **Инструкция `create_token`**:
    - Инициализирует минт (`mint`) для токена N-Dollar с 9 децималами.
    - Создает метаданные (`metadata`) для N-Dollar с помощью `mpl_token_metadata`.
  - **Инструкция `initialize_liquidity_pool`**:
    - Вызывает через CPI инструкцию `initialize_pool` программы `liquidity-pool` для создания пула ликвидности N-Dollar/SOL.
    - После инициализации пула чеканит `108,000,000` токенов N-Dollar напрямую в хранилище N-Dollar (`ndollar_vault`) созданного пула ликвидности. Авторитетом для минта выступает `authority`.
  - Определяет структуры аккаунтов `CreateToken` и `InitializeLiquidityPool`.

## 6. `programs/liquidity-pool/`

Программа, реализующая простой пул ликвидности для пары N-Dollar/SOL.

- **`Cargo.toml`**: Определяет метаданные и зависимости для программы `liquidity-pool`. Зависит от `anchor-lang` и `anchor-spl`.
- **`Xargo.toml`**: Файл конфигурации для кросс-компиляции Rust-кода под архитектуру BPF Solana.
- **`src/lib.rs`**: Основной исходный код программы `liquidity-pool`.
  - **Состояние (`Pool` PDA)**: Хранит авторитет, минт N-Dollar, ATA для N-Dollar (`ndollar_vault`), PDA для SOL (`sol_vault`), bump. Сиды PDA: `b"pool"`, `ndollar_mint.key()`.
  - **Инструкция `initialize_pool`**:
    - Инициализирует `Pool` PDA.
    - Создает ATA (`ndollar_vault`), принадлежащий `Pool` PDA.
    - Вычисляет адрес PDA (`sol_vault`) для хранения SOL, принадлежащего программе (сиды: `b"sol_vault"`, `pool.key()`).
  - **Инструкция `add_liquidity`**:
    - Позволяет пользователю внести N-Dollar и SOL в `ndollar_vault` и `sol_vault` соответственно.
  - **Инструкция `swap_sol_to_ndollar`**:
    - Пользователь переводит SOL в `sol_vault`.
    - Программа рассчитывает N-Dollar к выдаче по формуле `amount_out = (amount_in * balance_out) / balance_in`.
    - Переводит N-Dollar из `ndollar_vault` пользователю, подписывая `Pool` PDA.
  - **Инструкция `swap_ndollar_to_sol`**:
    - Пользователь переводит N-Dollar в `ndollar_vault`.
    - Программа рассчитывает SOL к выдаче по той же формуле.
    - Переводит SOL из `sol_vault` пользователю, подписывая `sol_vault` PDA (`invoke_signed`).
  - Определяет структуру состояния `Pool` и структуры аккаунтов `InitializePool`, `AddLiquidity`, `Swap`.
