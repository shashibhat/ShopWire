INSERT INTO products (
  sku,
  name,
  brand,
  category,
  color,
  size,
  price,
  original_price,
  stock,
  image_url,
  walmart_url,
  active
) VALUES
  (
    'nike-pg40-royal',
    'Nike Air Zoom Pegasus 40',
    'nike',
    'shoes',
    'royal-blue',
    '9',
    49.99,
    89.99,
    18,
    'https://i5.walmartimages.com/asr/example1.jpg',
    'https://www.walmart.com/ip/456789123',
    true
  ),
  (
    'nike-rev7-game',
    'Nike Revolution 7',
    'nike',
    'shoes',
    'game-royal',
    '9',
    38.00,
    55.00,
    5,
    'https://i5.walmartimages.com/asr/example2.jpg',
    'https://www.walmart.com/ip/987654321',
    true
  )
ON CONFLICT (sku) DO UPDATE SET
  name = EXCLUDED.name,
  brand = EXCLUDED.brand,
  category = EXCLUDED.category,
  color = EXCLUDED.color,
  size = EXCLUDED.size,
  price = EXCLUDED.price,
  original_price = EXCLUDED.original_price,
  stock = EXCLUDED.stock,
  image_url = EXCLUDED.image_url,
  walmart_url = EXCLUDED.walmart_url,
  active = EXCLUDED.active,
  updated_at = NOW();

